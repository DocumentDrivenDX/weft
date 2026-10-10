---
ddx:
  id: STP-009
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: TD-009
      kind: informed_by
    - id: TP-001
      kind: informed_by
---

# STP-009: security-compilation

## Story Reference

[[US-009-security-compilation]], [[TD-009-security-compilation]], [[SD-004-conformance-evolution]], TP-001.

## Scope and Objective

Allocate full FR-19–22 security acceptance without promoting component simulations.
Allocation validity is separate from executed acceptance. Every row remains
open for full product execution; foundation evidence is partial.

## Acceptance Criteria Test Mapping

| AC | Planned test | Assertion | Citation required | Layer | Execution state |
| --- | --- | --- | --- | --- | --- |
| US-009-AC1 | `security_transport` | Given pinned supported security sources, when the separate versioned transport is admitted, then exact custody and previous ordinary transport behavior survive without relabeling or fallback. | `@covers US-009-AC1` | Admission + lowering + native host | open; component-only foundation |
| US-009-AC2 | `security_admission` | Given selected policy and ontology meaning, when admission and composition run, then complete qualified identity and private-fact semantics are preserved or refused before emission. | `@covers US-009-AC2` | Admission + lowering + native host | open; component-only foundation |
| US-009-AC3 | `security_lowering` | Given a protected read, when logical and physical lowering run, then disclosure, replacement and protected predicate/order/group/join/aggregate semantics preserve null, absent and withheld distinctions. | `@covers US-009-AC3` | Admission + lowering + native host | open; component-only foundation |
| US-009-AC4 | `security_host_release` | Given a security artifact, when a native host executes it, then authenticated custody, current authority and guarded final release satisfy mandatory versioned obligations without inferred credentials or qualification. | `@covers US-009-AC4` | Admission + lowering + native host | open; component-only foundation |

## Executable Proof

Foundation command: `cargo test -p weft-core --lib --test security_admission --test security_literals --locked`.
Formal foundation commands and properties are defined in TD-009 and R5 evidence.
Full transport host parity, arbitrary complete admission, protected physical
lowering and native authority/release suites must be authored and executed before
these product criteria can pass. They are explicitly deferred beyond B-009;
no native backend access is presumed. No foundation pass closes these rows.

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
