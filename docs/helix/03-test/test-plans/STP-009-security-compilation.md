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

## Planned executable expansion for B-008 — 2026-10-10

The following suites are planned, not observed passes. Implement their exact
commands, fixtures and parseable citations before claiming coverage. Existing
foundation evidence remains partial; every full acceptance row above stays open.
The implementation plan owns S00–S12 ordering, not additional acceptance IDs.

| AC | Planned suites / slices | Additional positive and failure assertions |
| --- | --- | --- |
| US-009-AC1 | `security_binding_parity`, S00/S11/S12 | Exact selected source/request/response versions and source bytes; new security corpus through fresh CLI/native Python/Chromium; prior ordinary transport/parity; unknown/mixed version, stale pins, malformed/over-budget/corrupt whole envelope and no fallback. |
| US-009-AC2 | `security_graph_query_bridge`, `security_owner_requirements`, `security_capability_coverage`, S01–S04 | Original/candidate type separation; Record/Relationship namespaces, logical Keys and witness restrictions; correlated occurrences; complete rules/actions/context/facts at one cut; unknown or missing coverage on empty collections blocks; removed rule/action/constraint, fragmented primitive-feature capability unions, incompatible scan/action/application bundles and unknown extra selected meaning must refuse before backend invocation. |
| US-009-AC3 | `security_disclosure_codec`, `security_pgraw_lowering`, `security_truss_lowering`, `security_deltaraw_lowering`, `security_ashlar_lowering`, S03–S08 | All5 protected operator modes, eligibilityCOUNT/paging andSUM lineage; exact replacement domains, null/absence/withheld, ordered aliases/repeated outputs, bags and joins; bidirectional authority/query-carrier selected-Key populations and normalized values agree before filtering; extra/missing/duplicate carriers, same-typed component swaps and stale fields refuse; native rows/fields match independent oracle; unsupported transforms/domains and raw-value leaks refuse. |
| US-009-AC4 | `security_native_custody`, `security_authority_release`, S09/S10/S12 | Ordinary actor and excluded admin proof; source/inventory/fact/current-cut drift; direct/role/definer/retained-file/history/feed bypass controls; revocation acknowledgment follows guarded final release; old snapshots/cursors/publication leases refuse; cancellation/crash/timeout keeps uncertain custody and leaks no prefix. |

Every implementing test must cite its row using `@covers US-009-AC1` through
`@covers US-009-AC4` as applicable. Compiler tests and native tests are separately
identified. A positive internal lowering experiment never counts as public
activation/native authority proof. Privacy controls cover private facts and
derived diagnostics/count estimates across every selected host/native sink.

S00 maps the authoritative shared132-case ledger to actual owners and required
profile subsets; no local test renames or deletes a shared obligation. Native
profiles require actual original actors, reviewed independent oracles and
CONTRACT-053 fresh-execution/assertion/source/inventory evidence. Stored receipt
replay, synthetic graph tables and observer flags cannot substitute for real
Truss/Ashlar execution. Shared write/effective-date/history/feed cases remain
host/backend prerequisites where required; Weft protected-read tests do not
claim to implement those writes.

Final qualification retains both ordinary workspace compositions, module/spec
checks, seven source mutation controls and original2,181-case host regression,
plus a separately authored positive security corpus and all mandatory native
cases. Missing executable, missing citation/assertion, skip, timeout, stale
source, incomplete inventory or unresolved review means open/failed. A changed
formula requires new witnesses, broken controls and actual implementation
correspondence; B-009 R5 does not certify new physical behavior.
