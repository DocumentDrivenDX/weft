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

Owner-directed UMF security work adds CONTRACT-007 and the versioned 0.5 request
foundation. Existing ordinary transports remain core07. The separate core08
source-custody path and required security source transport refuse activation
until semantic admission, logical/physical lowering, disclosure result domains
and authenticated native host obligations are implemented. B-008 is open;
foundation refusal tests cannot close security/native acceptance. Rust/Python/
browser parity and all four UMF backend plans remain required.


### B-008 immutable compiler mapping handoff — 2026-10-08

Implemented the actual Rust owner's private-constructor profiled-query export,
`weft.security.mapping-handoff/0.1.0`, under CONTRACT-007. It preserves the
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
CONTRACT-007. UMF's `tools/security/weft-handoff-probe.py` retains two actual
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


## B-009: Reliability remediation

### Scope and governing authority

Address the seven findings from the owner-requested reliability evaluation.
FR-10–12 and CONTRACT-003 govern embedding fidelity, bounded input and evidence;
FR-19–22 and CONTRACT-007 govern the security foundation. Architecture and
TP-001 retain compiler/storage/host ownership and scoped native qualification.
This plan applies HELIX 0.15.4 modularity, Rust configuration, OpenTelemetry
diagnostics and formal-methods guidance, read from the installed HELIX 0.15.4 full-plugin catalog. The active
catalog is its workflows/graph.yml; templates are not copied into this repository.

The repository owner authorized implementation, Astra Ultra review, commits and
pushes. Each chunk must receive an independent gpt-6-astra review at ultra
reasoning, resolve material findings, and receive a follow-up disposition before
commit and normal push to codex/reliability-review. No force push or production
deployment is included. Preserve unknown source content and historical receipts.

The existing uncommitted B-008 security foundation is a prerequisite snapshot.
The isolated checkout retains its exact starting bytes; its review and focused
tests precede a checkpoint commit. Security remains blocked before composition
or emission. This remediation does not activate protected queries, invent native
authority, or complete security physical lowering. FR-19–22 remain separately
allocated open product requirements until their full acceptance evidence exists.
The goal closes the evaluation findings, not those additional product outcomes.

### Shared constraints

- Record scope, commands, results, source fingerprints, reviewer model/reasoning
  and residual limits for each chunk under 04-build/evidence/reliability/.
- Keep the original checkout intact. Include prerequisite source only after its
  explicit review; do not stage unrelated user changes or copy secrets.
- Preserve frontmatter and artifact IDs; contracts own exact shared surfaces,
  technical designs own wiring/properties and test plans own coverage.
- Keep IO/configuration/telemetry in runners and host entry points. The Rust core
  remains deterministic, offline and independent of storage implementations.
- Qualified ordinary compiler profiles never qualify security, production
  layouts, current stored data, new engines or released packages.
- A failed or missing check is recorded as failed/unknown, never as passed.

### Implementation slices

| Slice | Responsibility | Depends on | Validation and review gate |
| --- | --- | --- | --- |
| R0 | Review/checkpoint existing B-008 source; frame US-008 reliability and US-009 security allocations, TD-008/009 and STP-008/009; adopt concern scopes | Plan review | Astra Ultra reviews the plan first; exhaustive PRD requirement → story → AC → test-plan allocation; delete requirement/story/AC controls; focused original/draft security tests and prerequisite diff review |
| R1 | Bound CLI input before allocation; stable IO/UTF-8 refusal, no partial artifact; immutable Catalog input access | R0 | Real CLI tests for exact limit, overflow with unread suffix, invalid UTF-8/read failure; compile-negative privacy control and public API regressions |
| R2 | Define actual package/type ownership, construction/integration map and enforced dependency graph | R1 | Cargo metadata dependency checker; allowed edge, forbidden edge and cycle controls use the real checker; compiler visibility proves private access refusal; local/pre-commit/CI use one command |
| R3 | Centralized validated harness configuration and pinned toolchain wrapper | R2 | Defaults/file/env/CLI precedence and malformed/missing values; no default ops handles; mutation runner works with configured Cargo and isolated paths; unapplied mutation, missing test and optimized-Python controls refuse; no compiler ambient configuration |
| R4 | Safe bounded runner diagnostics and read-only retrieval with pinned OTel mapping | R3 | Redaction before sinks, stdout cleanliness, record/retention limits, timeout/cancel/failure/final outcome; bounded retrieval and expired cursor behavior; actual local OTLP receiver/mapping test, unavailable exporter/local capture failure/queue overflow/flush timeout controls; no duplicates or invented trace IDs |
| R5 | Explicit security acceptance allocation and formal specification/correspondence | R0, R2 | Stable requirement/property IDs; precise semantic review plus independently executable finite checks with success/failure witnesses and broken controls; exact tool/bounds/assumptions/source mapping; security activation gate remains closed |
| R6 | Fresh bindings in CI and reproducible current-source qualification | R1–R5 | Fresh CLI/Python/Chromium whole-artifact parity against retained ordinary qualified corpus and blocked security cases; raw-resource/corruption controls; both valid Rust workspace compositions; feature conflicts; historical receipt replay and current-source stale controls; final Astra Ultra review and terminal hosted CI success at the final pushed SHA |

### Design and test realization

US-008 allocates the seven findings to independently measurable acceptance
criteria. TD-008 owns reliability wiring and configuration/diagnostic boundaries;
STP-008 owns commands and negative controls. US-009 allocates FR-19–22 without
claiming completion; TD-009 owns the selected security formal specification and
STP-009 distinguishes existing foundation tests from deferred native activation
and host enforcement gates. The existing 30 acceptance IDs and fixtures remain
unchanged. specs:check requires exhaustive PRD requirement → story → acceptance criterion →
test-plan allocation without hardcoding the old total. Every requirement and
story criterion must have an allocation; deletion of a requirement mapping, a
story allocation or a criterion fails. Explicit open/deferred execution remains
distinct from allocation validity and passing executed acceptance.

The module map covers core, both adapters, runtime composition, Python/WASM
bindings, browser wrapper, test probes and reliability tooling. The checker
validates actual Cargo package edges and cycles. Intra-crate type ownership and
semantic coupling receive named review evidence; unsupported static checks are
explicit. No directory exclusion or inflated baseline hides a new edge.

Harness configuration owns executable/output paths, timeouts, capture/export
limits and non-secret fingerprints. Injected operational resource handles have
no committed defaults. Rust compiler requests remain explicit inputs. Existing
fixture-native executors keep host-owned credential access and are outside
the local runner's automatic execution; no native database call is authorized
merely by a config file or selected model.

Diagnostics use safe typed lifecycle events and referenced evidence. Default
subprocess capture excludes raw streams; supported safe summaries are allowlisted.
Run manifests distinguish pending/failed/completed runs, loss and crash limits.
The runner owns local files/retention, stderr console and an optional bounded
OTLP route; no deployed service, production sidecar or service SLO is introduced.
Formal checks cover compiler/security-foundation properties; publication/current
authority are documented host assumptions with deferred implementation evidence.

Historical B-007 evidence retains original source hashes. Its verifier separates
immutable receipt/source custody from present-workspace qualification: historical
source is verified through retained snapshots or Git object identity, never
rewritten to match current files. Current qualification uses a fresh manifest
covering the complete compiler/binding/schema/corpus/oracle/test/harness/build/
lock/config/checker input set and exact freshly loaded binaries. Relevant changed,
missing or newly introduced files invalidate it. The old source custody verifier
resolves its declared f81565a1addaa6d2c83561f62d3805d1167233ee checkpoint from
Git; CI checkout fetches that object and fails if it is unavailable.
Semantic artifact mismatch invalidates current qualification.
Replaying a native receipt never becomes a new database execution claim.

### Issue decomposition and review loop

The runtime goal tracks R0–R6; no external tracker is required. Each chunk has
one diff, commands/evidence and an Astra Ultra review. Material findings are
implemented and the changed scope re-reviewed until no blocking finding remains.
Record advisory/deferred findings with requirement authority and rationale.
Only then commit its explicit paths and push normally. R6 validates the complete
committed composition and the final current-source evidence. The compiler
workflow must trigger on codex/reliability-review pushes (or be explicitly
dispatched at the final SHA). Record hosted run identity, head SHA and required
job outcomes; missing, skipped, cancelled or failed required jobs keep the goal
incomplete. Every final source/review fix requires a new final-SHA CI result. Review evidence
must remain attributable to the reviewed source; a later change reopens affected
checks. A reviewer timeout or unavailable model remains an incomplete gate.

### Risks and rollbacks

| Risk | Response | Rollback |
| --- | --- | --- |
| Pre-existing security source is incomplete | Review only the blocked foundation; preserve open activation/native gates | Decline the checkpoint or revert the reviewed change on this branch |
| Public Rust Catalog access changes | Add read-only getters and update callers; compiler-negative and semantic regression checks | Revert the encapsulation chunk without weakening transport pins |
| Historical evidence cannot be tied to original source | Fail historical custody and retain the unknown; obtain exact Git blobs/snapshots | Keep historical claim suspended; never rewrite hashes |
| Long property/domain suites | Focused tests per chunk, complete both compositions at final gate; capture exact terminal outcome | Do not release/qualify an incomplete run |
| Telemetry leaks source or subprocess content | Allowlist before every sink; inject secret-shaped controls and test each sink | Disable export/capture and record loss without changing compiler behavior |
| Formal abstraction misses runtime behavior | Reviewed property-to-code/test mapping and explicit exclusions | Withdraw the assurance claim; keep activation closed |

### Exit criteria

All seven findings have named implementation/acceptance evidence and no unresolved
blocking Astra Ultra review findings. Original source and historical evidence are
preserved. Fresh committed-source ordinary compiler checks and actual Python/
Chromium parity pass; security refusals retain exact versions and no partial SQL.
All completed chunks are committed and pushed to the intended branch; the final
pushed SHA has terminal successful hosted CI with no missing/skipped required
jobs. Native
production/security execution and package release remain separately open and
are explicitly excluded from these reliability support claims.
