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


### Mapping preparation transport prerequisites — 2026-10-13

US-009-AC1/AC2 retain component-only qualification. `security_mapping_transport` tests in the original Rust core derive all5 actual operator uses across two resolved scans, retain primary/separate action closures, refuse unknown caller IR/action overrides/source drift/invalid transports, preserve new profile bytes as unauthenticated source inputs, and bound repeated parameter payload before substitution. The CLI test verifies original UTF-8 and the exact byte ceiling.

`scripts/security/check-mapping-transports.py` verifies the actual imported extension path and16 original native Python/CLI controls. `scripts/security/check-mapping-browser.ts` verifies16 actual Chromium WASM controls, all16 response hashes identical to native Python/CLI and no external requests. Both cite US-009-AC1/AC2 as prerequisites. Their authored fixture is generated by `scripts/security/mapping-transport-fixture.py`; its opaque mapping is explicitly unqualified. Independent review adds exact integer/Unicode carriers, malformed/nested duplicate/version controls and the repeated-parameter refusal.

Required backend continuation is unchanged: consume the original handoff under authenticated installed profile/mapping custody; qualify native SQL/parameter correspondence, actual roles/hidden facts and disclosed/original/prohibited operators; refuse incompatible/missing original-action dependencies before SQL, including empty results; preserve original guard/settlement/release and source-drift obligations. No preparation, transport or structural test closes native B08 or these full product criteria.

Merged-owner integration controls retain a useful ordinary authored-field query on a model containing a valid relationship, while refusing actual relationship shorthand with `WFT-SECURITY-QUERY-UNSUPPORTED` and rejecting the new complete-entity read profile at the original transport schema. These prevent silent expansion of the selected security subset when ordinary compiler capabilities evolve.


### Explicit native Python relationship candidate bridge — 2026-10-15

The separate `truss-security-relationship-candidate` Cargo feature in runtime and
Python exposes `prepare_truss_security_relationship_candidate_json(request)` and
`inspect_truss_security_relationship_result_json(request)`. Both invoke the same
original Rust producer/inspector used by the separate relationship binaries.
Python accepts and returns original JSON strings; refusals raise ValueError with
original diagnostic code/message. The existing 4MB producer and 32MB inspector
input bounds, closed envelopes, exact producer-byte reconstruction, query/policy/
unvalued Subject parameter channels, codecs and result obligations are unchanged.
No new protocol, installed backend, caller identity or execution callback is added.
Ordinary builds, including the ordinary PG18 feature, omit these two methods.
Existing security mapping and compiler registrations remain closed to this profile.

Fresh macOS arm64/Python3.12.14 installed-wheel checks compare exact Rust/Python
candidate and inspector outputs for both original permitted and observation-denied
sources. SQL/newline/source/version/authority overrides, missing source/terminal
scan declarations, mismatched result hashes, duplicate JSON keys, malformed and
oversized envelopes refuse. Empty supplied batches only exercise correspondence:
they do not establish native population completeness, source truth, original
operand capture, current authority, authenticated Subject or guarded publication.
This binding increment promotes no original backend case and does not activate
public security compilation. Native enforcement and full B-008 remain open.
