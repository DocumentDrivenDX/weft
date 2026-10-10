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

Implement the Weft compiler governed by the PRD, CONTRACT-001 through CONTRACT-004, TD-001 through TD-006 and TP-001. The bootstrap delivered specifications and fixture validation. B-001 now passes its bounded embedding gate; ADR-001 selects Rust for the foundation. The spike is not the public compiler; ADR-002 now accepts the structural backend boundary; public contracts remain draft specifications with scoped executable evidence. No production storage compatibility is claimed.

## Shared Constraints

Input is Weft SQL plus explicitly supplied pinned UMF modules; output is target SQL with parameters, types, decoding, obligations, versions and diagnostics. One registered backend per query. Initial targets are Truss/PostgreSQL and Ashlar/Databricks. No database IO in the compiler core, implicit model fetching, native SQL passthrough or silent meaning loss.

## Implementation Slices

| Slice | Story / area | Depends on | Validation gate |
|---|---|---|---|
| B-001 | SPIKE-001 native Python/browser | None | Pin toolchain; actual extension and browser compile/refusal parity; decide ADR-001 |
| B-002 | US-001 model resolution, parser, typed IR | B-001 | Independent oracle, corpus semantic/refusal tests; finalize versioned Rust IR representation |
| B-002A | US-007 versioned application-read frontend | B-002 | CONTRACT-004, independent member/presence/order/count/relationship/parameter fixtures |
| B-003 | US-002 registry, binding validation, capabilities | B-002A | Third synthetic plugin; exact typed trait and manifest contract; finalize ADR-002 |
| B-004 | US-005 compile envelope and embeddings | B-003 | Original and application-read requests across Rust/Python/browser; precise UTF-8 spans and deterministic diagnostics |
| B-005 | US-003 Truss adapter | B-003, documented UMF storage realization and mapping (pinned drafts permitted) | Native PostgreSQL exact results, storage boundaries and emitted host obligations |
| B-006 | US-004 Ashlar adapter | B-003, accepted Ashlar layout/binding | Native Databricks exact results, numeric/collation/publication boundaries |
| B-007 | US-006 qualification and release matrix | B-004–006 | All 30 ACs, expanded corpus, property/fuzz/mutation gates and versioned evidence |

## Issue Decomposition

No tracker items are created by this bootstrap. The slices are ready for work-item creation with references to the corresponding US, TD, STP and this plan. Use dependency links above; keep each issue scoped to a failing acceptance test and its implementation. Backend-owner mapping decisions block production qualification; owner-authorized candidate adapter development and fixtures proceed against exact versioned drafts.

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

B-001: complete for its bounded scope; [evidence](evidence/B-001-native-python-browser.md). B-002: complete for the 0.1 frontend; [evidence](evidence/B-002-frontend.md). B-002A: complete for its versioned application frontend scope; [evidence](evidence/B-002A-application-reads.md). B-003: complete for the registered library boundary; [evidence](evidence/B-003-backend-interface.md). B-004: complete for the public envelope and native Python/browser component scope; [evidence](evidence/B-004-compile-embeddings.md). B-005: complete for its owner-authorized candidate compiler scope. [Acceptance audit](evidence/B-005-acceptance.md) maps US-003-AC1–AC4 to original model/binding admission, independent native PostgreSQL results and actual host transport/publication checks. The original sales query passes all 16 row/props home cuts; numeric comparison prerequisites reject hidden out-of-domain values. Recursive/presence, complete entity, count, relationship and composite paging evidence is retained in the [execution record](evidence/B-005-native-preparation.md). The sales driver passes 224 actual psycopg cases, including 192 execution/publication refusals. The latest implementation checkpoint passes 160 Rust regressions and fresh Python/Chromium parity. Capabilities stay candidate; native null is unsupported by the selected profile. Truss runtime adoption and installed-production qualification are separate. No production or released package claim is made. B-006: complete for its owner-authorized candidate compiler scope. The [acceptance audit](evidence/B-006-acceptance.md) maps US-004-AC1–AC4 to native Databricks exact results, explicit props/typed homes, relationship and recursive/presence boundaries, and actual buffered host transport/custody checks. The final Ashlar package passes 18 Rust tests; the workspace checkpoint passes 182. Compound values pass 133 native cases, six name/resource controls and 48 entity/keyset combinations; the native host driver passes 30 cases plus 49 independent host controls. Fresh Python/Chromium/CLI builds match 463 artifacts with an explicitly recorded declaration-only correction. Capabilities remain candidate; no production policy, storage adoption or package release is qualified. B-007: compiler acceptance complete; final CI/review/merge pending. The [30-criterion acceptance matrix](evidence/B-007-acceptance-matrix.json) maps each P0 outcome to retained independent frontend, native, host and refusal evidence. The unchanged 636 initial cases, 1,200 expanded relational assertions, 10,000 deterministic properties, 5,000 parser properties and seven detected source mutants meet TP-001's floors. Finite semantic-family ledgers account for supported/refused paths without claiming every private branch or model/query cross-product. Separately versioned Supported registrations retain exact native domains and host obligations; historical candidates remain separate. Fresh native CPython/Chromium builds preserve 2,181 full artifacts with 4,362 audited joins, seven resource cases per host and eight semantic corruption controls. The [final workspace record](evidence/B-007-workspace-qualified-final/summary.json) passes 241 candidate and 242 qualified tests, each across 35 suites, with none ignored or filtered. The [support inventory](evidence/B-007-support-inventory.json) names exact versions, domains and execution conditions. License, registry ownership and final distribution artifacts remain gates before distribution.

## PR #2 scope update

B-002 establishes the unchanged 0.1 frontend. The next separately reviewed slice B-002A adds the owner-requested application-read extension before backend contracts are finalized. B-003 through B-007 must include its typed representations, capabilities, host obligations and native tests. A production backend cannot advertise complete entity/relationship/paging support without corresponding binding evidence.

## Compiler/storage ownership correction (2026-10-07)

The user clarified that UMF owns logical and physical schema meaning, Truss supplies its storage realization and mapping, and Weft owns query lowering, target SQL and result decoding. Truss runtime implementation, public decoder adoption, authorization procedures and installed-production qualification are not prerequisites for compiler implementation or completion of the compiler slice. Hosts execute SQL and enforce the emitted obligations. B-005 uses exact pinned draft storage descriptions plus independent PostgreSQL fixtures; production compatibility remains separately qualified. This supersedes earlier runtime/adoption blocking interpretations without weakening exact type, presence, storage-home or unknown-meaning refusal requirements. Full recursive and scalar query coverage, public backend integration and actual native/compiler evidence remain required B-005 work. The goal resumes in B-005; B-006 follows its merge.


## B-008: Shared security compiler integration

Owner-directed UMF security work adds CONTRACT-006 and the namespaced security 0.1 request
foundation. Main ordinary transports retain core07/core08 source admission. The separate core08
source-custody path and required security source transport refuse activation
until semantic admission, logical/physical lowering, disclosure result domains
and authenticated native host obligations are implemented. B-008 is open;
foundation refusal tests cannot close security/native acceptance. Rust/Python/
browser parity and all four UMF backend plans remain required.


### B-008 immutable compiler mapping handoff — 2026-10-08

Implemented the actual Rust owner's private-constructor profiled-query export,
`weft.security.mapping-handoff/0.1.0`, under CONTRACT-006. It preserves the
original resolved application plan and complete source/field-use/scan-action
obligations, with exact reuse refusal and deterministic bounded JSON. The owner
test validates retained resolved-plan meaning, derived protected-field action,
separate action obligations, source-byte/backend-version refusal and repeat
export equality. Current-source Rust admission replay passes, with canonical
UMF overlays and ordinary compiler regressions intact. UMF evidence resides at
`/Users/erik/.codex/worktrees/1598/umf/docs/helix/04-build/evidence/security/weft-admission.json`.

This implements a concrete backend-mapping handoff component, not physical
lowering or host authority. Public compiler security activation, Python/browser
transport, native field-use enforcement, physical fact issuer and final-release
qualification remain open. No acceptance criterion is credited by packet shape.


### Actual Rust handoff inspection transport — 2026-10-08

Added the offline-buildable `weft-core` example `security_mapping_handoff` under
CONTRACT-006. UMF's `tools/security/weft-handoff-probe.py` retains two actual
compiler packets and nine execution observations, including seven refusals for
protected query use, stale backend/model sources, unknown version/member,
duplicate JSON and over-budget input. Refusals have no stdout packet. Exact
source/schema inputs and binary digest are retained in UMF evidence
`docs/helix/04-build/evidence/security/weft-handoff.json`. This gives physical
mapping tests an executable original-owner source, not fabricated annotations.
No backend is activated, no data query runs and no trusted host issuer is claimed.


### Preserved core primary-Key metadata — 2026-10-09

SecurityOntologyClosure accepts the supported core Boolean primary annotation,
checks carriers and at-most-one primary, and keeps explicit ontology keyId
selection. Original source bytes/trees retain true and false. A distinct Resource
secondary Key over integer salary cannot replace explicit string resourceId pk;
IR endpoint and Resource identities remain pk. Null/string/numeric primary
carriers on selected and unselected Keys refuse at WFT-MODEL; duplicate primary
annotations refuse ontology closure. Rust source admission and unsupported
activation remain separate. UMF retains actual original compiler evidence in
weft-admission.json. This is a source-fidelity integration increment, not draft
0.2 admission, native Key enforcement, or protected execution qualification.


### B-008 main integration — 2026-10-10

The security foundation is integrated against main14c5814 while preserving the incoming95991ee browser/evidence update. Main's CONTRACT-005 authored-path meaning and ordinary compile0.3/0.4 remain unchanged; the security contract and transports have an explicit [identity migration](evidence/security-main-integration/identity-migration.json). `cargo test --release --locked --workspace` passes all 556 tests across 44 groups, with none failed, ignored or filtered. The [checkpoint](evidence/security-main-integration/checkpoint.json) retains exact source/log hashes, scoped Astra ultra review, the corrected initial diagnostic failures and the intentionally interrupted debug attempt. Both security transports remain blocked before physical emission. B-008, installed Python/browser parity and the original native backend acceptance plan remain open.


### B-008 typed field semantics — 2026-10-10

Under CONTRACT-006 and UMF's independent requirement issuer design, private
OwnerSourceDemands retains every selected Field/Context event's exact borrowed
qualified field reference and original catalog field carrier. Stored events also
retain the exact inventory owner reference and raw ontology field declaration,
including protection/query-use settings. Context is a distinct typed channel
without a stored-field classification. No domain normalization or model cloning
is used to manufacture applicability; facets, allowed values and opaque content
remain available through the original carrier. Catalog/ontology lookup visits and
retained source-ID copies are charged. Source-only derivation keeps its prior
ledger. Field/Context event-key projection must equal the payload inventory.

Independent controls cover actual catalog/ontology pointers and authored fixture
keys, stored/context uses of the same Boolean field, equal-but-foreign reference,
carrier, owner and classification substitutions, exact/minus-one retention limits,
and populated 4096/4097 entry boundaries. The [execution evidence](evidence/security-field-events/checkpoint.json)
retains actual runs, declared source custody and scoped Astra review. Three
conditional algebraic address laws preserve channel, stored owner and original
occurrence; erasure controls and independent fresh-context replay accompany them.
These laws do not prove Rust pointer/string representation or traversal refinement.

This completes field payload custody for the existing Field/Context event subset.
Key/member order, query projection/operator/output details, complete backend
kind/template applicability, authenticated profiles and original native assertion
mappings still require issuance work. B-008 and all132 original backend cases
remain binding; no native acceptance or public security lowering is promoted.

Final current-source `cargo test --release --locked --workspace` passes557 tests
across44 groups with zero failures/ignored/filtered. All822 declared repository
inputs were frozen before execution and verified unchanged at terminal exit0.
External Cargo registry/build runtime dependencies are outside that inventory.
Astra's read-only implementation review is clean after channel-only/self-join
controls, and independently replays all9 current formulas in fresh Z3 contexts.
The checkpoint preserves both pre-refinement successful runs and the corrected
initial formal parser failure; stored receipts are not relabeled or repinned.
