---
ddx:
  id: STP-008
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: US-008
      kind: informed_by
    - id: TD-008
      kind: informed_by
    - id: TP-001
      kind: informed_by
---

# STP-008: reliability

## Story Reference

[[US-008-reliability]], [[TD-008-reliability]], [[SD-004-conformance-evolution]], TP-001.

## Scope and Objective

Prove B-009 reliability outcomes on the complete reviewed compiler revision.
Allocation validity is separate from executed acceptance. The allocation rows record planned test ownership; executed acceptance is retained
separately in the B-009 chunk receipts. A planned row alone never claims a pass.

## Acceptance Criteria Test Mapping

| AC | Planned test | Assertion | Citation required | Layer | Execution state |
| --- | --- | --- | --- | --- | --- |
| US-008-AC1 | `current_qualification` | Given retained historical receipts and a changed revision, when a maintainer qualifies support, then complete input custody and fresh binary/artifact evidence distinguish historical observations from current qualification. | `@covers US-008-AC1` | Rust/host/tooling/CI | planned |
| US-008-AC2 | `bounded_host_inputs` | Given hostile or malformed input, when CLI or library hosts admit it, then bounded reads and immutable prepared-source custody prevent unbounded allocation, stale parsed meaning and partial artifacts. | `@covers US-008-AC2` | Rust/host/tooling/CI | planned |
| US-008-AC3 | `module_boundaries` | Given changed package dependencies or private access, when boundaries are checked, then actual forbidden edges/cycles refuse and ownership/construction/integration responsibilities are inspectable. | `@covers US-008-AC3` | Rust/host/tooling/CI | planned |
| US-008-AC4 | `configured_mutations` | Given a configured toolchain and mutation experiment, when the harness runs, then validated configuration and fail-closed detection distinguish actual killed mutations from missing tests, unapplied edits and optimized Python. | `@covers US-008-AC4` | Rust/host/tooling/CI | planned |
| US-008-AC5 | `safe_diagnostics` | Given runner success, failure, timeout or diagnostic outage, when evidence is captured and retrieved, then bounded safe events identify outcomes/loss without exposing raw source, credentials or subprocess output. | `@covers US-008-AC5` | Rust/host/tooling/CI | planned |
| US-008-AC6 | `exhaustive_formal_allocation` | Given governed requirements and security foundations, when allocation/formal assurance is checked, then omitted requirements/criteria fail and precise bounded properties, witnesses and correspondence remain distinct from deferred native acceptance. | `@covers US-008-AC6` | Rust/host/tooling/CI | planned |
| US-008-AC7 | `fresh_host_ci` | Given the final pushed compiler revision, when CI qualifies the ordinary subset, then fresh Python and Chromium artifacts match the full reference artifact corpus and every required job completes successfully at that revision. | `@covers US-008-AC7` | Rust/host/tooling/CI | planned |

## Executable Proof

R0: `bun run specs:check` and `bun test scripts/spec-allocation.test.ts`.
R1: runtime input tests and compiler-negative Catalog access. R2: real Cargo
metadata boundary checker and broken-edge/cycle controls. R3: configured mutation
runner controls under ordinary and optimized Python. R4: runner lifecycle,
privacy, retention/retrieval and actual local OTLP receiver tests. R5: pinned
formal checker with SAT witnesses and UNSAT safety checks plus broken controls.
R6: both Rust compositions, fresh CLI/native wheel/Chromium full artifact parity,
resource/corruption checks and successful hosted CI at final pushed SHA.
Exact commands and source fingerprints are retained per B-009 chunk; tests must
cite the corresponding row when implemented. Planned commands are not passes.

## Data and Setup

Pinned source and toolchain, authored independent fixtures, exact reference
artifacts and explicit native execution authority where applicable. Retained
native receipts are historical evidence; replay is not a new database execution.

## Edge Cases and Failure Modes

Delete an allocation, alter custody, remove a test, corrupt an artifact, omit a
required CI job or supply incomplete evidence: refuse qualification rather than
accept a missing gate. Selected unknown security meaning remains blocked.

## Build Handoff

Follow B-009 dependency order and independent Astra Ultra review per chunk.
Done requires named executable tests, citations, observed passes and no unresolved
blocking review. Deferred full security/native acceptance remains open.

## B-009 observed execution

R0–R5 reviewed receipts are retained under `04-build/evidence/reliability/`. They
record actual allocation controls, bounded CLI/immutable Catalog checks, module
gates, clean-baseline mutations, SDK receiver/privacy/loss/retrieval checks, and
bounded formal plus actual Rust correspondence. R6 current-host receipts and
input manifest are separate from the historical B-007 acceptance matrix. The
final hosted CI run at the pushed SHA is required for AC7; absent/failing/skipped
required jobs keep current acceptance incomplete. FR19–22/US009 native security
acceptance remains open.
