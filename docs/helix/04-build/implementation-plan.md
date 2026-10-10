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


### B-008 selected Key/member semantics — 2026-10-10

Under CONTRACT-006 and the shared independent requirement issuer design, private
owner Key/KeyField events now retain the original selected Key definition,
qualified target and exact original key-ID/member-slice references. Member events
retain their zero-based ordinal, original native member declaration, qualified
inventory reference and catalog field carrier. Existing canonical source strings
retain their one-based member positions; no source string is decoded to recover
meaning. A selected nonprimary Key remains selected when another Key is primary.
Raw definition metadata, member order and native carrier meaning remain borrowed.

The dedicated private key-events module resolves exact selected IDs and complete
ordered native member references, charging every candidate/member comparison.
Retention checks identity rather than structural equality, with exact/minus-one
visit/text controls and a populated4096/4097 cardinality boundary. Key/KeyField
source-key projection equals the complete payload map. Assignments=None preserves
the legacy source-only ledger and avoids this resolution/retention work.
Independent actual compound fixtures cover both member orders, exact original
pointers and selected-vs-primary identity. Isolated controls cover target, Key ID,
member-slice, definition, ordinal, field-reference, carrier, native declaration
and namespace substitutions; missing/duplicate/reordered/shortened/foreign selected
Key definitions refuse. [Execution evidence](evidence/security-key-events/checkpoint.json)
retains initial compiler integration errors and the fixture's mandatory-name
admission failure, followed by actual corrected runs.

Three conditional algebraic laws cover member ordinal, selected Key identity and
original occurrence. Nine formulas retain pre-solve SMT and four declared source
hashes frozen before construction/rechecked before publication. Astra ultra's
read-only review is clean and independently replays all9 in fresh Z3 contexts.
These are constructor/projection laws, not Rust representation/traversal refinement
or native key enforcement; simplified erasure controls are not Rust mutants.

Next retain typed query projection/operator/output semantics and complete
independent backend kind/template/site/failure/prerequisite and original assertion
mappings with authenticated exact profiles. Complete issuance, physical lowering,
B10/admission-drift closure and integrated native qualification remain unfinished.
All132 original cases remain binding; this checkpoint promotes none. B-008 and
the active goal remain open.

Final current-source `cargo test --release --locked --workspace` passes559tests
across44groups, zero failed/ignored/filtered. All823 declared repository inputs
are prefrozen and verified unchanged at terminal exit0. External Cargo registry
and build-runtime dependencies remain outside that inventory. Both initial failed
attempts and the prior source-specific formal receipt remain preserved. The current
four formal source pins match; Astra independently replays all9 current formulas.
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

## B-008 continuation: Protected-read security completion plan — 2026-10-10

### Scope and governing artifacts

Plan the remaining FR-19–22 / US-009-AC1–AC4 work under FEAT-006,
CONTRACT-001–006, architecture, ADR-001/002, TD-009 and STP-009.
This is a planning increment; implementation and native operations are not started.
B-009 is complete at `2b88a9add31c9f6de32f7c3c68f9546c1b497fa0`, with
[successful hosted CI](https://github.com/DocumentDrivenDX/weft/actions/runs/38027179951).
Its receipts qualify that checkpoint; subsequent document/source changes do not
inherit its current-input manifest. Preserve those receipts unchanged.

Shared meaning comes from the security worktree at
`/Users/erik/.codex/worktrees/1598/umf`: CONTRACT-052-security-semantics,
CONTRACT-053-security-enforcement, SD-008, TD-055–057, STP-055–057,
SPIKE-009/010, and `03-test/security/cases.json`. This is explicitly not the
unrelated CONTRACT-052/053 domain-pack scope in the current UMF checkout.
Current remote Weft main `9104e55` is a divergent integration baseline: ordinary
compile0.3/0.4 paths and governed IDs (including CONTRACT-005) have meanings
that differ from this branch. Integration must preserve current main contracts,
resolve versions and artifact IDs explicitly with a recorded old-to-new mapping,
and obtain fresh whole-composition qualification before reuse. The branch
checkpoint is not a qualification of that future merge.
S00 must pin the actual upstream revision, original files and schema digests;
sibling draft content is planning input, not an approved published version.
The inspected shared ledger has 132 required cases / 28 criteria and 106 errors;
its failed receipt is historical evidence, not fresh acceptance or a readiness gate.

Weft owns source admission, query requirements, logical/physical lowering and
result decoding. UMF owns policy/ontology meaning. Truss/Ashlar and raw-profile
hosts own native mappings, original actors, installed inventory, authority facts,
publication and final release. Backend/host work is an explicit external
prerequisite, not a new storage implementation inside the compiler.
Protected reads are the Weft scope. Shared write/lifecycle tests remain mandatory
for native profiles that claim them, but this plan does not invent Weft write SQL.

### Baseline and shared constraints

Reuse the existing original 0.1 admission/evaluation/query-profile/mapping-handoff
path and separately typed draft 0.2 source/ontology/type/IR/dependency/composition
and raw/opaque/Record-backed graph simulations described in CONTRACT-005.
Draft simulation and supplied completeness/trust claims remain conditional.
Candidate dependency extraction exists; candidate actual-query/scan integration,
whole-collection enforcement, backend result matching and native cuts remain gaps.
No stage may convert a candidate plan into an original admitted plan by relabeling.

- Preserve ordinary 0.1/0.2 transports, exact core07/core08 pins and blocked 0.3
  behavior. Select new request/response/schema versions in S00; do not assume
  draft security 0.2 is released or widen the blocked-only response in place.
- Preserve full qualified Key identity, directed endpoint roles, witness owner,
  lexical occurrence/correlation, source bytes and unknown archives. Opaque
  witnesses permit incidence/existence only; identity/attribute/count/distinct
  operations on them refuse. Unsupported selected meaning refuses atomically.
- Authenticate complete authority cuts outside the core. Incomplete facts,
  unknown rule truths, failed constraints or a missing obligation refuse the
  whole operation before any SQL/result release; no admitted row prefix survives.
- Keep original null, explicit absence, withholding and typed replacements
  distinct. An output envelope is possible-value coverage, never permission.
  Preserve ordered aliases/repeated outputs; projection dependency sets are not
  output columns. COUNT retains every scan/action; SUM retains argument lineage.
- Keep compiler IO/configuration/credentials/telemetry out of core. Use
  CONTRACT-008 typed operational handles, pinned tools, bounded runners and safe
  diagnostics. No implicit warehouse, schema, actor, fact issuer or test workspace.
- Apply the existing module map and `bun run modules:check` locally, through the
  optional hook and CI. Update ownership/allowed edges before new feature modules;
  forbidden import, cycle, public-constructor and source-substitution controls
  stay mandatory. Semantic coupling remains independently reviewed.
- Formal evidence names property, authority, state, assumptions, bounds,
  solver/version, SAT witnesses, UNSAT checks, broken controls and actual Rust/SQL
  correspondence. Existing finite composition proofs do not prove new graph,
  query, disclosure, native-currentness or liveness behavior.
- Safe host diagnostics reuse verified SDK/receiver machinery. New protected
  paths must prove privacy across errors, explain/count estimates, logs, traces,
  retrieval and native diagnostics; existing runner tests do not qualify those
  paths. Raw policy/facts/SQL/results/credentials never enter diagnostic sinks.

### Implementation slices and dependencies

Each row is an independently reviewable gate. Split a row further by operator,
profile or interface when its diff cannot remain bounded; keep its dependency.
Planned tests below are unimplemented unless explicitly covered by retained
foundation evidence; their names do not establish coverage.

| Slice | Work / owner | Depends on | Required validation before closeout |
| --- | --- | --- | --- |
| S00 | Reconcile upstream source/result/coverage/matching contracts and schemas; Weft + UMF owners | B-009 checkpoint | Pin original authority files; reconcile current main version/artifact-ID collisions and integrated baseline; choose exact supported versions and migration/refusal; allocate all four Weft ACs and all132 shared cases to actual owners; missing/changed upstream/schema or allocation refuses. Record activation rollback and diagnostic contract before wiring. |
| S01 | Adopt security module boundaries and immutable requirement construction; Weft | S00 | Real module checker and allowed/forbidden/cycle controls; compile-negative tests prohibit caller-created admitted requirements and plan substitution; no host/SDK/solver dependency in core. |
| S02 | Bridge draft source/graph plans to actual resolved query, scan/actions and collection admission; Weft | S01 | `security_graph_query_bridge`: raw, opaque and Record-witness variants; self-joins, nested correlation, field-freeCOUNT, original-action uses, empty/unused populations and whole-cut incompleteness; exact keys and context channels; public activation still blocked. |
| S03 | Derive owner requirements and conservative result coverage from actual rules, uses and ordered outputs; Weft | S02 | `security_owner_requirements`: all permit/require/forbid branches, every action/operator, repeated aliases, all transform classes and source domains; conservative empty-envelope refusal; work/text charges before allocation. Model coverage independently and replay real Rust counterexamples. COUNT/SUM stay explicit unresolved requirements until proved, never omitted. |
| S04 | Interpret closed capability/constraint/obligation matching and versioned result-domain declarations; Weft + UMF | S03 | `security_capability_coverage`: coherent complete scan/action and whole-application bundle coverage with compatible selected capabilities; reject fragmented primitive-feature unions, incompatible bundles, unknown/extra constraints, stale source/session/profile and candidate substitutions before executable physical lowering; only the explicit security 0.2 trusted registration factory may run once after owner admission (see current-main clarification); no opaque declaration/evidence-ID matching. `security_disclosure_codec`: original-null/absent/withheld/replacement domains, whole-batch corruption, ordered outputs and computed results; no executable public security result yet. |
| S05 | Protected relational lowering against explicit raw PostgreSQL mapping; Weft + raw host | S04 | `security_pgraw_lowering`: permit/require/forbid, private correlated facts and bag semantics; disclosed vs separately authorized original predicate/order/group/join/SUM; row eligibility forCOUNT/page; exact native types, errors and no raw-value leakage. Before filtering, prove bidirectional authority/query-carrier selected-Key population and normalized-field correspondence; extra/missing/duplicate carriers, same-typed Key-component swaps and stale values refuse. Compare actual SQL/decoder results to independent oracle using least-privilege actors. |
| S06 | Truss physical mapping and protected lowering; Weft + Truss | S05 | `security_truss_lowering`: original node/edge/property homes, logical Keys, direction and Record/opaque witnesses; hidden fact/retained-carrier bypass controls. Actual Truss mapping/native evidence required; synthetic graph tables cannot qualify this profile. |
| S07 | Raw Delta protected lowering/publication contract; Weft + raw Delta host | S04 | `security_deltaraw_lowering`: native policy/compute/table support, typed masking/operators, exact carriers, immutable data cut and publication eligibility. Refuse unsupported native authority guarantees; PostgreSQL evidence never certifies Delta. |
| S08 | Ashlar physical mapping and protected lowering; Weft + Ashlar | S07 | `security_ashlar_lowering`: actual graph/typed homes, logical identity and private-fact/publication mappings; original warehouse/actor/inventory observations and bypass controls, independent from raw Delta fixtures. |
| S09 | Original-actor/fact/inventory admission and change invalidation, per profile; host/backend owners | S04 + corresponding S05–S08 | `security_native_custody`: real ordinary connections, excluded assessor/admin controls, complete issuers/coverage and bidirectional selected-Key population/value correspondence to every query carrier before filtering, role/definer/direct/file/history/feed bypass closure, mapping/role/source drift, failed installation preserves prior protection. No asserted flag or caller-signed packet grants authority. |
| S10 | Current authority and guarded final release, per profile; host/backend owners | S09 | `security_authority_release`: participating writer/read guards, revocation drain/ack barriers, stale snapshot restart/refusal, temporal lease, buffered/stream/cursor/cache paths, publication retirement, cancel/crash/rollback/timeout and abandoned resources. Observe lock/commit/release barriers, never sleep-only order. Unsupported paths refuse before output. |
| S11 | Public host/binding fidelity for admitted versioned security behavior; Weft + host owners | S04 + corresponding S05–S10 | `security_binding_parity`: fresh CLI/native Python/real Chromium full-envelope and decoded-result parity, malformed/source-mutation/trap/cancellation/resource controls; no subprocess Python fallback, browser IO or compiler authentication claim. Pre-activation compilation remains available only through explicit internal qualification boundaries. |
| S12 | Per-profile acceptance, atomic activation and final integrated qualification; all owners | S00–S11 for selected profile | Re-execute complete allocated Weft tests and every required shared case for the claimed profile; no missing/skip/timeout/unknown/incomplete assertion. Fresh source/binary/inventory custody, independent native assessor and Astra Ultra review; switch activation only for exact fully enforced supported tuple. Final hosted CI at final pushed SHA plus separately recorded native run IDs; rollback/drift withdraw support without weakening old protection. |

S05→S06 and S07→S08 are two independent target families after common compiler
coverage. Native S09/S10 may be designed early but cannot qualify a profile before
its actual lowering and mapping pass. S12 may qualify one explicitly bounded
profile while others remain blocked. Completion of **all remaining work** requires
all four backend plans and every mandatory shared outcome, not merely the first
vertical slice. No claim may borrow another profile's native evidence.

### Validation plan and acceptance allocation

STP-009 owns exact AC-to-test mapping and shared-case citations. S00 replaces
placeholder runner declarations with reviewed argv-array commands, independent
oracles, expected assertion IDs, source fingerprints and bounded timeouts as
implementation becomes available. Null/unimplemented runners keep acceptance
open. Native reports identify actual actor/build/session/inventory, protected
input cut, expected/observed typed rows and failed attempts without secrets.

Every chunk runs the applicable pinned Rust suites, `bun run specs:check`,
`bun run modules:check`, relevant fail-closed/mutation controls and independent
review. Changes reopen affected formal/model/native correspondence and historical
claims stay at their original checkpoint. Full composition qualification runs
both candidate/qualified workspaces, conflicts, fresh bindings and current input
closure. Ordinary2,181-case parity remains a regression gate and does not stand
in for a newly authored positive security corpus.

### Execution contract and continuation evidence

This turn authorizes planning only. No new implementation goal, native connection,
credential read, installation, production migration or activation is started.
After implementation authorization, review the plan with Astra Ultra; implement
small chunks, resolve/re-review material findings, then commit/push each completed
chunk if authorized. A source change always requires affected verification and
new final-SHA CI. Native operations require an explicitly selected disposable or
existing admitted test resource and owner-approved actor/administrative scope.

Record plan revision, completed slice IDs, command/result/source/reviewer evidence,
open owner decisions, failed attempts and next action under
`04-build/evidence/security/`. The first execution action is S00: inspect and pin
upstream contracts and close version/ownership/native-resource decisions. It is
not removal of the unsupported compiler guard.

### Open decisions, risks and rollback

| Decision / risk | Owner and action before dependent work | Safe response |
| --- | --- | --- |
| Draft0.2 source/result/matching grammar not published | UMF + Weft owners select exact schemas/version pair in S00 | Retain separate candidate types and old blocked-only response. |
| Relationship query frontend cannot express required original meaning | Weft + UMF owners resolve an explicit typed bridge/version under CONTRACT-001/005 | Refuse relationship helpers; never fabricate endpoint fields. |
| Four backend registrations and external writer/lifecycle ownership | Backend + host owners record exact profiles/owners/required132-case allocation in S00 | Unowned obligations remain blocked; no generic catch-all receipt. |
| Available PostgreSQL/Delta test actors and least-privilege installation scope | Operator selects admitted resources before native execution; no values in plan | Continue pure compiler work; mark native qualification blocked until real access exists. |
| Aggregate/disclosed-mode semantics or result-domain facets incomplete | Weft owner proves COUNT/SUM/operator meaning under S03–S05/S07 | Keep explicit unresolved requirements/refusals; do not weaken domain to scalar family. |
| Incomplete revocation/publication/bypass participation | Host/backend owners inventory every reachable path/writer in S09/S10 | Refuse profile activation; retain prior admitted protection. |
| Proof/model differs from Rust or SQL | Independent assessor reviews mapping and executes weakened controls | Withdraw corresponding claim and block dependent release. |

### Exit criteria

Plan readiness requires ordered slices, authorities, owners, negative controls
and explicit decisions; it does not approve a schema version or native profile.
Execution closes US-009 only after all four criteria have actual cited passing
transport/admission/lowering/native-release evidence, all required shared cases
for the declared scope pass freshly, no material review finding remains, and
final pushed source has successful required CI. Full shared write/native product
acceptance is separately owner-qualified; it cannot be inferred from Weft reads.


### Main integration compatibility decision (2026-10-10)

The owner explicitly approved preserving main's existing versions and assigning
security its own version. Ordinary compile 0.3 / SQL 0.3 nullable reads and the
separate compile 0.4 / 0.4.1 path entrypoints keep their existing schemas and
behavior. Blocked security moves from the branch's compile 0.3 / SQL 0.2 to
compile 0.5 / SQL 0.2, with new request/response schemas. Security still refuses
before backend composition or executable emission. No old version is repurposed.

Main's CONTRACT-005 authored two-hop paths retains its ID and contents. The
branch security CONTRACT-005 is explicitly migrated to CONTRACT-007; current
security links and gate inputs follow that mapping. Historical receipts at
2b88a9a retain their original IDs, versions and byte hashes. They qualify their
original checkpoint only. Fresh integrated qualification remains pending.

The generic weft-runtime CLI preserves main's exit-2, fixed stderr-code, empty
stdout transport-error protocol. The R1 unbuffered OS-handle sentinel guarantee
applies to that binary; separate weft-paths and weft-paths-keys retain their
existing bounded readers and are not qualified for exact unread OS offsets.
Module ownership now includes main's arithmetic, paths, keys and distribution
tooling. Inventory checks require semantic review of the added APIs; refreshing
that inventory alone is not architectural qualification.


Main qualification now has an independently built, clean reference at
14c58146dad2e3aeb755c8035e91754a150be74c. All 2,181 retained requests are
unchanged and integrated responses match that main reference exactly. Of these,
2,128 complete response objects are unchanged from historical native-qualified
receipt objects; 53 changed outputs carry current-main compatibility evidence only.
The complete differences, source hashes, build features and compiler hashes are
recorded under evidence/main-integration-20261010. Changed cardinality SQL and
diagnostic phases do not inherit native qualification. Fresh cross-host parity
and final hosted CI are still required before merging this integration.


### Concurrent main security reconciliation — 2026-10-10

Main ed2b677 adopts separate security0.1/0.2 protocols and expanded pure security components under CONTRACT-006. The [Astra Ultra-approved integration plan](evidence/main-ed2-integration-20261010/astra-approved-integration-plan.md) governs this reconciliation. [Identity migration](evidence/main-ed2-integration-20261010/identity-migration.json) moves the runner contract006 to008 and preserves main security 006 and blocked compatibility 007. Old receipts and the [original approved continuation plan](evidence/main-ed2-integration-20261010/prior-approved-security-plan.md.original.txt) remain historical.

This current-main clarification supersedes the prior S04 blanket callback prohibition: compatibilitycompile0.5 andsecurity0.1 never dispatch a backend; generic or ordinary-factorysecurity0.2 refuses BACKEND-REQUIRED with ordinary callback 0; the valid explicitsecurity0.2 registration entrypoint calls the trusted factory once and still refuses LOWERING-UNSUPPORTED with SecurityBackend::lower0; owner-admission failures call registration 0. Security transport 0.2 still consumespolicy/ontology0.1; selected candidatesource0.2 remains a separate logical type. No result checker or simulator authorizes native execution or data release.

Prepared Catalog source remains immutable. Main’s faithful opaque-JSON readers and evolved evaluator/result semantics remain; resource checks reserve normalization capacity before allocation, charge cloned identities and share composition ledgers across repeated rows. Simulation retains 4M normalized/16M copy limits; separate result phases retain 16M budgets, with explicit finite phase boundaries rather than a reset per row/action. Native authentication, complete source/coverage/authority, physical lowering and release qualification remain open.

S00 must reconcile the pinned current UMF main cd4cc937 security authority CONTRACT-062/063 (migrated052/053), preserving the original 132-case owner allocation. Incoming 556 Rust passes remain historical component evidence. New independent current-main comparison, merged-source tests, Astra chunk review, fresh 2419-plus input custody and final-SHA hosted CI are pending; the earlier 014ae44 green CI cannot qualify this newer compiler.

### B-009 current-main reconciliation — 2026-10-10

Astra Ultra approved the dedicated security protocol reconciliation at `202b0b9`
and the subsequent typed field/Key custody merge at `4bf8d2d`. Main's ordinary
versions and dedicated security 0.1/0.2 remain intact; compatibility compile 0.5
is a separate blocked-only entrypoint. Runner CONTRACT-006 migrated explicitly
to CONTRACT-008, preserving main's security CONTRACT-006 and compatibility
CONTRACT-007. Original plans and migration preimages remain retained.

The [new independent baseline](evidence/main-ed2-integration-20261010/baseline/baseline.json)
pins clean main `bf64c8c` and compares all 2,181 unchanged requests with the merged
compiler. All complete response objects match under type-preserving canonical
JSON. Exactly 2,128 also match the historical native-qualified response objects;
53 changed outputs remain open for native requalification. Historical wire bytes
were not compared. The older baseline and native receipts remain unchanged.

Current focused checks pass 224 core library and 64 security admission tests,
including public API controls for aggregate mask normalization and escaped-copy
exhaustion across rows. Those limits return `WFT-LIMIT/result`; smaller batches
pass both cells and selection. Main's field/Key conditional algebra scripts
replay 18 formulas, without claiming Rust refinement or native enforcement.
Module ownership, offline pinned schema validation and CLI boundary controls
pass. Fresh CLI, installed Python and real Chromium checks now require ten blocked
security vectors alongside seven resource controls, with exact blocked response
members and separate CLI input-limit refusal. Fresh full-input custody and final
hosted CI are required before merge; this paragraph does not claim their completion.

### B-009 read-only diagnostic CLI dependency repair

An intermediate hosted run timed out during optimized diagnostic CLI startup at
its existing five-second process limit. The CLI imported the telemetry SDK despite
using only bounded record readers. Following Astra Ultra plan review, unchanged
record validation, cursor authentication and retrieval helpers moved into the
standard-library-only `diagnostic_records` module. The SDK writer reexports those
same objects and retains real SDK logging, tracing and encoding. Retrieval keeps
its one-second, byte, record, owner and privacy limits. Actual successful `-S`
and malformed-cursor `-O -S` CLI controls retain the five-second harness deadline;
no timeout or validation check was removed.
