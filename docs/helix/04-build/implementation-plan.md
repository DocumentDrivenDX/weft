---
ddx:
  id: weft.implementation
  type: implementation-plan
  activity: build
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.architecture
      kind: informed_by
    - id: TP-001
      kind: informed_by
    - id: ADR-001
      kind: informed_by
    - id: ADR-002
      kind: informed_by
---

# Build plan

## Scope

Implement the Weft compiler governed by the PRD, CONTRACT-001 through CONTRACT-003, TD-001 through TD-006 and TP-001. The bootstrap delivered specifications and fixture validation. B-001 now passes its bounded embedding gate; ADR-001 selects Rust for the foundation. The spike is not the public compiler; ADR-002 and public contracts remain draft/proposed. No production storage compatibility is claimed.

## Shared Constraints

Input is Weft SQL plus explicitly supplied pinned UMF modules; output is target SQL with parameters, types, decoding, obligations, versions and diagnostics. One registered backend per query. Initial targets are Truss/PostgreSQL and Ashlar/Databricks. No database IO in the compiler core, implicit model fetching, native SQL passthrough or silent meaning loss.

## Implementation Slices

| Slice | Story / area | Depends on | Validation gate |
|---|---|---|---|
| B-001 | SPIKE-001 native Python/browser | None | Pin toolchain; actual extension and browser compile/refusal parity; decide ADR-001 |
| B-002 | US-001 model resolution, parser, typed IR | B-001 | Independent oracle, corpus semantic/refusal tests; finalize versioned Rust IR representation |
| B-002A | US-007 versioned application-read frontend | B-002 | CONTRACT-004, independent member/presence/order/count/relationship/parameter fixtures |
| B-003 | US-002 registry, binding validation, capabilities | B-002A | Third synthetic plugin; exact typed trait and manifest contract; finalize ADR-002 |
| B-004 | US-005 compile envelope and embeddings | B-002A | US-007 versioned application-read frontend | B-002 | CONTRACT-004, independent member/presence/order/count/relationship/parameter fixtures |
| B-003 | All 636 requests across Rust/Python/browser; precise UTF-8 spans and deterministic diagnostics |
| B-005 | US-003 Truss adapter | B-003, accepted Truss storage binding | Native PostgreSQL exact results, storage boundaries and host obligations |
| B-006 | US-004 Ashlar adapter | B-003, accepted Ashlar layout/binding | Native Databricks exact results, numeric/collation/publication boundaries |
| B-007 | US-006 qualification and release matrix | B-004–006 | All 24 ACs, expanded corpus, property/fuzz/mutation gates and versioned evidence |

## Issue Decomposition

No tracker items are created by this bootstrap. The slices are ready for work-item creation with references to the corresponding US, TD, STP and this plan. Use dependency links above; keep each issue scoped to a failing acceptance test and its implementation. Backend-owner mapping decisions block production adapter slices while frontend and fixture-plugin work can continue.

## Validation Plan

- Write failing behavioral tests from independently specified cases before implementation.
- Preserve fixture expectations; resolve disagreements through contract review, never by copying emitted SQL into expected results.
- Require relevant checks and actual evidence before closing a slice.
- Update canonical specifications when behavior changes; keep unsupported operations explicit.

## Risks and Rollbacks

Rust packaging/toolchain feasibility has bounded B-001 evidence on one platform; broader wheel/browser matrices remain unqualified. Missing backend layouts block claims, not frontend progress. Keep adapters separately registered so an unsupported profile can be disabled without changing logical semantics. Version incompatible changes; preserve prior corpus and evidence.

## Exit Criteria

All 30 P0 story criteria pass; both initial adapters have qualified native profiles; native Python and real browser embedding pass; support inventory names exact versions and domains; no candidate-only claim is advertised as supported. Release procedures, package ownership and license are decided before distribution.

## Execution status

B-001: complete for its bounded scope; [evidence](evidence/B-001-native-python-browser.md). B-002: complete for the 0.1 frontend; [evidence](evidence/B-002-frontend.md). B-002A and B-003 through B-007: not implemented. Story acceptance allocations remain planned; B-001 does not close US-005 or any release conformance gate.

## PR #2 scope update

B-002 establishes the unchanged 0.1 frontend. The next separately reviewed slice B-002A adds the owner-requested application-read extension before backend contracts are finalized. B-003 through B-007 must include its typed representations, capabilities, host obligations and native tests. A production backend cannot advertise complete entity/relationship/paging support without corresponding binding evidence.
