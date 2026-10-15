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

### B-008 query payload custody — 2026-10-10

Under CONTRACT-006 and the shared independent requirement issuer design, private
Projection/QueryField/Operator/Output events retain original scan inventories,
qualified references, catalog carriers, ontology classification declarations,
admitted requirements and the full original resolved application Plan. Projection
events retain the exact resolved projection. Operator requirements borrow the
profile-owned admitted QueryUse and preserve Disclosed versus OriginalAuthorized
with its original action string. Ordered outputs retain original position, alias
and expression through the actual requirement; repeated fields remain distinct
outputs. Query uses/projections are resolved families, not every syntactic AST
occurrence. No source-string decoding or manifest-inferred semantics is introduced.

Retention charges visit/text work, refuses conflicting pointer custody and bounds
population at4096. Source-key projection equals the complete query payload map;
legacy source-only traversal retains its existing ledger. Actual tests independently
assert six source IDs, exact references, repeated outputs, original-action custody,
self-join scan isolation, exact/minus-one budgets and4096/4097 limits. Isolated
substitutions cover field, operator and output identities. Astra ultra's read-only
review identified a test expectation using the raw query's use pointer rather than
the admitted profile-owned clone; corrected pointer plus separate value equality
and three additional isolated controls leave no remaining implementation findings.
The failed test attempt is retained with corrected runs in the
[execution checkpoint](evidence/security-query-events/checkpoint.json).

This is tested private custody preparation, not a formal Rust refinement proof,
complete independent requirement issuer, authenticated backend profile, public
security lowering or native enforcement qualification. Earlier source-specific
formal receipts are preserved without repinning to this changed traversal. All132
required backend cases remain binding and none is promoted. Next complete the
independent backend kind/template/site/failure/prerequisite and original assertion
mappings, remaining association custody, authenticated issuance, physical lowering
and B10/admission-drift closure. B-008 and the active goal remain open.

### B-008 original association custody — 2026-10-10

Under CONTRACT-006 and the shared independent requirement issuer design, private
Association events retain exact scan/action inventories, qualified association
references, original native catalog records and complete original ontology
declarations. Ordered endpoint roles, targets and component fields remain in the
borrowed declaration, along with selected Key identity and classification metadata.
This preserves a per-scan/action association dependency, not one payload for each
Exists AST occurrence. Rule occurrences separately retain those expressions.

Declaration lookup charges every candidate and uses full qualified identity;
missing/duplicate matches refuse. Identity retention is charged before insert,
refuses foreign equal objects and bounds population at4096. Association source-key
projection equals the payload map. Source-only traversal retains its existing
ledger. Actual tests independently bind each source ID to its qualified reference,
verify exact original inventory/catalog/declaration pointers, self-join ownership,
both compound endpoint orders, original empty inventories, each of five pointer
substitutions, exact/minus-one budgets and populated4096/4097 boundaries. Astra
ultra's read-only review caught a golden test-oracle gap that could accept swapped
payloads; independently authored source-to-reference expectations close that gap.
The final review has no remaining findings.

The [execution checkpoint](evidence/security-association-events/checkpoint.json)
retains actual core/workspace results and qualified source custody. This private
semantic input does not complete backend kind/template/site/failure/prerequisite
and assertion mappings, authentication, physical lowering or native qualification.
It adds no formal Rust refinement proof and promotes none of the original132
required backend cases. B10/admission-drift closure and integrated native acceptance
remain open. B-008 and the implementation goal remain active.



### B-008 independent template expansion — 2026-10-10

Under CONTRACT-006 and UMF's independent backend requirement issuer annex,
private template expansion now instantiates independently authored kinds, original
contracts/sites/failures/cases and bounded same-occurrence prerequisites from
actual owner event categories and structural rule payloads. It retains complete
shared origins and independently assigned deployment duties even for zero-edge
selections. It does not read manifest obligations to author the expected inventory.
The actual matcher then compares that independently expanded inventory against
original declaration custody; codec/privacy fragments cannot satisfy a whole scope.

The typed applicability subset covers18 owner event categories and all existing
Rule variants, not payload-specific field/domain/operator/transform/backend rules.
The declared full profile case set and catalog completeness remain trusted premises.
Exact backend catalogs, original132-case mappings, authenticated registration,
physical lowering and native enforcement remain required. Prerequisites currently
instantiate only at the same selector/address; cross-target mappings refuse.
Separate template and rule-enumeration budgets do not establish aggregate CPU/
memory bounds. Public security lowering remains closed. No new formal Rust
refinement or original acceptance-case promotion is claimed; B-008 remains open.


The [execution checkpoint](evidence/security-requirement-templates/checkpoint.json)
retains a terminal release workspace run:569 passing tests across44 groups,
including226 core tests and nine new decisive controls. All828 declared repository
build/test inputs were frozen before execution and verified unchanged afterward;
external Cargo registry/runtime dependencies are excluded. Astra independently
verified all pins, full anchored test summaries, all nine exact passing lines and
the retained log hash. No actionable review findings remain within the stated
subset. Specification checks pass for51 artifacts/31 schemas/30 compiler criteria/
636 fixtures, separately from security acceptance. Development failures and their
corrections are retained; exact failed-attempt source snapshots were not captured,
so those logs do not support source-qualified claims. The goal remains active.


### B-008 original required-case catalog binding — 2026-10-10

Under CONTRACT-006 and UMF CONTRACT-063, catalog custody now preserves the exact
147,740-byte original132-case plan with explicit source SHA256 and selected
backend/registration premises. Each selected backend retains all12 shared and30
backend cases, their complete original assertions/procedures and unresolved
metadata. Catalog-bound template expansion requires this exact42-case inventory,
even for selected zero-edge deployment duties. S10's additional assertion and
B10's counterexample/absent procedure stay intact. No command is executed and
all native qualification remains pending. The ordinary private trusted-case path
and historical receipts retain their original meaning.

Expected plan/registration hashes and backend association are not authenticated.
The test catalog's template-to-case assignments are synthetic, not independently
qualified production mappings. Separate catalog, template and rule phase bounds
do not prove native/Rust refinement or aggregate resource bounds. All original132
cases remain binding, no acceptance case is promoted, and B-008 remains open.

Validation checkpoint: the frozen workspace run passed 573 tests across 44 groups (230 core), with all 833 declared input pins unchanged and zero failed, ignored, measured, or filtered tests. Astra ultra independently audited the terminal receipts and four new test results with no blockers. See [retained checkpoint](evidence/security-case-catalog/checkpoint.json). All 42 selected backend cases remain pending; no authentication or native enforcement acceptance is promoted.


#### B-008 authored deployment qualification duties

The next private catalog revision replaces the test-only42-case catch-all with an
explicit independently authored qualification interface for each original required
case. All42 distinct duties, exact owner/site/failure/case/prerequisite tuples and
all authored capability assignments are checked before the stricter versioned
DeploymentIssued result is created. This includes unchosen and zero-edge capabilities.
The original-case record supplies the full assertion; catalog membership alone
cannot discharge it. The four backend homes retain separate qualified IDs and
implementations despite shared assertion interfaces.

Five new controls compare the complete map to a separately authored42-row golden
fixture, remove every duty and every capability assignment, substitute each contract
component, retain actual owner source/scope/kind inventory and test a coherent
manifest/profile catch-all that passes weaker instance matching but refuses the
strict gate. Exact169-visit/21504-reserved-text and minus-one controls isolate the
shared deployment comparison ledger. Development core execution passed235 tests;
full source-frozen workspace validation and independent review must be retained
before landing. Semantic payload applicability, trusted profile authentication,
native bodies/evidence and public lowering remain required. Historical required
acceptance remains26/132; no selected case is promoted by this catalog.

Final [retained checkpoint](evidence/security-deployment-catalog/checkpoint.json):578 release workspace tests across44 groups,235 core, all five new controls passing, all837 declared input pins unchanged, zero failed/ignored/measured/filtered. Astra ultra independently checked the source mapping, bounded gate and terminal receipts with no remaining blocker in the candidate-duty scope. Specification checks passed51 artifacts/31 schemas/30 criteria/636 fixtures. The distinct strict result remains private and all42 cases remain pending.


#### B-008 typed payload applicability candidate

Template0.2 now derives mandatory scalar/nullability/refinement-family/protection,
key, query/output and ordered association duties directly from actual borrowed
payloads under CONTRACT-006. Independently authored source/address/scope golden
expectations check33 payload occurrences and48 extra scoped requirements in the
repeated-output fixture. Each applicable selector and its eligible origin is
omitted in turn; every omission refuses. Legacy template0.1 rejects payload
selectors. Whole0.2 exact/minus-one visit/text controls remain conditional bounded
execution checks, not a formal implementation refinement proof. Astra's output
availability finding was applied before freezing: required and absent-allowed
values remain distinct; unestablished values refuse.

Scalar/facet families and occurrence presence are qualified by these component
checks only. Numeric refinement compatibility, constant/transform literal and
domain meaning, complete authenticated template profiles, optional/nullable result
admission, physical/native implementation and original acceptance qualification
remain open. Historical required acceptance stays26/132; the goal remains active.
Frozen workspace receipts and independent review must be retained before landing.

Additional controls execute actual-owner context/operator/ordered-association
dispatch across two scans and two actions. Authentic resolved source-plan helpers
check COUNT/SUM and optional Field output typing before result admission. An exact
direct-original result contract passes for a required Field and refuses for the
otherwise equivalent absent-allowed Field; this does not forbid all withheld
optional-output source plans or establish aggregate result admission.

Final [retained checkpoint](evidence/security-payload-applicability/checkpoint.json):
584 release workspace tests across44 groups,241 core, all six new controls passing,
all840 freshly frozen declared input pins unchanged, zero failed/ignored/measured/
filtered. Astra ultra independently audited source semantics,184 Rust/Cargo path
coverage and all terminal receipts with no landing blocker in the stated subset.
Specification checks passed51 artifacts/31 schemas/30 criteria/636 fixtures.
Development logs retain corrections as diagnostics without failed-attempt source
snapshots; only the final fresh frozen run supports source-qualified test claims.
No original required acceptance case is promoted and the full goal stays active.


#### B-008 typed rule domain/literal demand extraction

Template0.3 adds actual borrowed Field/Context/Constant operand and constant0.1.0
transform domain/literal duties under CONTRACT-006. Earlier template experiments
retain their meaning. Three direct controls cover all five wrapper families and
typed absence, required-null/wrapper/transform refusal, independent Boolean
4-visit/7-text and null3-visit/0-text pre-encoding ledgers, and identical disclosure
values at distinct ordered addresses. An actual-owner golden authors28 exact
source/scope/kind/address duties across both Equal operands and two repeated
constant transforms (distinct disclosed target Fields, identical output Field/value)
despite a populated false condition. Duplicate target disclosure in one source
rule remains refused by original type admission; the direct repeated-target
primitive test does not widen source admission. Every applicable
selector and origin omission refuses, as do legacy rule selectors and whole0.3
ledger minus-one limits. These tests qualify demand extraction, not literal
normalization or native representability. Frozen workspace execution and
independent review are required before landing. All132 acceptance cases remain
binding; historical required acceptance stays26/132 and the full goal stays active.

Final [retained checkpoint](evidence/security-rule-payload/checkpoint.json):588
release workspace tests across44 groups,245 core, all four new controls passing,
all842 freshly frozen inputs unchanged, zero failed/ignored/measured/filtered.
Astra ultra independently audited source/pin/log custody,184 Rust/Cargo paths
and all terminal results with no remaining landing blocker in the conditional
demand-extraction scope. The original duplicate target admission guard remains
unchanged. Specification checks passed51 artifacts/31 schemas/30 criteria/636
fixtures. Development correction logs are diagnostic only without failed-source
snapshots; the final fresh run provides source-qualified evidence. No numeric/native
compatibility, authenticated profile, Rust/database refinement or original required
acceptance case is promoted; the full goal remains active.

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


### B-009 private requirement-template and catalog reconciliation

Main commits `ae13dbb` and `f7a682e` add private typed requirement-template
issuance and original required-case catalog custody. Their complete source,
132-case catalog, provenance, conditional claims and historical checkpoint
receipts are preserved. Both private modules and their host checkpoint scripts
are explicitly owned in the module map. The new coverage fixture uses the
immutable Catalog getter. Main's B-008 execution notes remain together before
the B-009 reliability plan. Public security lowering remains closed; no
authentication, native enforcement or acceptance-case promotion is introduced.

The previous `01bccbd` PR and push workflows passed for the preceding source.
They cannot qualify these newer private compiler modules. Fresh source-bound
formal checks, rebuilt CLI/Python/Chromium parity and full new input custody,
Astra Ultra review, and terminal hosted CI at the final pushed head are required
before main integration. The fixed independent ordinary baseline and the 53
open native-requalification outputs retain their original scope.


### B-009 typed deployment and payload reconciliation

Main commits `60e8255`, `c6d8c53` and `5120c54` add private exact authored
deployment duties, typed payload applicability and typed rule domain/literal
verification duties. Their sources, catalogs, qualification limits and historical
checkpoint receipts are preserved. The module map owns both new private modules
and all three checkpoint runners. Existing immutable Catalog access, aggregate
result-phase ledgers and blocked public security protocols remain in place.

Both workflows passed at preceding head `0db5a32`; they cannot qualify this new
source. Astra Ultra review, fresh source-bound formal and host/full-input custody
checks, and final-head hosted CI are required for the reconciled source. Native
security enforcement, authentication, full acceptance and the 53 historical
output requalification gaps remain open.


### Current security-envelope fallback regression — 2026-10-12

Current maincdd8fa6 already contains the merged security compiler foundation.
The stale sibling owner checkout retains the historical compile0.3/0.4 spike;
it must not overwrite current ordinary contracts or be represented as current
integration. The current separate security envelopes are
weft-security-compile/0.1.0 and0.2.0; compile0.5 remains blocked-only under
CONTRACT-007. Successful semantic/registration checks still do not emit native
security lowering or authorize execution.

`evidence/security-envelope-no-fallback-20261012/receipt.json` retains the
current source checkpoint, pinned offline Rust1.90.0 release command and logs.
All64 original security admission tests pass. With the new five-variant envelope
regression all65 pass, zero failed/ignored. Historical compile0.3/0.4 with selected
security content, unknown ordinary/security versions and empty version all block
without ordinary composition or partial sql/parameters/logicalPlan emission.
No compiler implementation changes or product SQL allowlist are introduced.

Trace FR-19/20/22 and CONTRACT-007 source/version/fallback obligations; shared
US-056 AC7/AC10 prerequisites. This does not qualify installed Python/browser
parity, full workspace, B-009, actual physical lowering/ontology enforcement or
any original backend acceptance case. Broader pending reviews and duties retain
their independent scope. The original shared implementation goal remains active.


### Optional original presence correspondence — 2026-10-12

The result declaration checker now represents possible explicit absence beside
Original for absent-allowed fields. The supplied-fact result selector compares
value/null/explicit-absence exactly; all six mismatched cross-state combinations
and missing presence metadata refuse. The earlier incomplete optional declaration
control still refuses. Public compilation continues to refuse lowering/emission.

`evidence/security-optional-presence-20261012/receipt.json` retains source archive,
source pins and fresh Rust1.90.0 release verification:256 unit +66 security
admission +3 literal tests pass, zero failed/ignored. The source change after that
run is a comment clarifying the truth-only selector's scope; executable inputs
are identical. Three Z3 4.15.4 queries replay in fresh contexts: two UNSAT for
conditional presence correspondence/missing-fact refusal and one SAT showing
truth-only checks cannot establish presence. No Rust/native refinement is claimed.
Trace CONTRACT-007 original field presence; shared STP-056 R09/B07/B09/B15,
US-056 AC6/AC7 prerequisites. These remain simulated facts; authentication,
native lowering/codec/current policy/publication and all original backend cases
remain separate requirements. The shared132-case goal stays active.


### Optional presence under active withholding — 2026-10-12

`evidence/security-optional-withholding-20261012/receipt.json` retains24 actual
owner fact-selection controls: active/false Withheld × value/null/explicit absence
× four candidate cells. Under active Withheld every world accepts only the exact
same value-free Withheld cell; Original value, Original(null) and Absent refuse.
A false mask preserves the matching Original value/null or explicit Absent and
refuses Withheld. Conservative declarations retain all potential outcomes in both
policy variants. Public compiler emission remains blocked.

All67 Rust security admission tests pass, zero failed/ignored; core sources are
byte-identical to the preceding325-test component qualification. The test setup
was factored without changing its source semantics. Three Z3 4.15.4 queries and
fresh-context replays give two UNSAT/one SAT: active-mask tag equality, false-mask
presence preservation and an absence-first leaking countermodel. This is finite
supplied-fact/tag analysis, not Rust/native/SQL or error/timing noninterference.
Trace CONTRACT-007 and shared STP-056 R08/R09/B07, US-056 AC6 prerequisites.
Authenticated native facts, installed masks, protected observation/current authority
and final release remain open. No original backend case or complete profile is
promoted; the full shared implementation goal remains active.


### Physical declaration shape and metadata budgets — 2026-10-12

The compiler now checks physical declaration structure through its original owned
security context. Positive empty-installation/empty-string parameter and boundary
controls pass; unknown version, whitespace/NUL/oversized SQL, duplicate operation
ID,513 installations, noncontiguous parameter positions, nonobject provenance,
nullable slots, oversized values/parameter vectors and excessive metadata depth
refuse. Exactly512 installations and16384 aggregate slots pass; the next complete
operation exceeding the slot limit refuses. Aggregate SQL/metadata budget refusal
is independent of per-operation limits.

Opaque JSON numeric tokens now consume their original lexical byte budget before
serialization; a65-byte number fails a32-byte ledger and passes an exact65-byte
ledger without coefficient expansion. This fixes a missing accounting branch in
the shared declaration visitor, without changing opaque meaning.
`evidence/security-physical-shape-20261012/receipt.json` retains original source
pins/archive and Rust1.90.0 release verification:257 unit +68 security admission
+3 literal tests,328 passing total, zero failed/ignored. Shape success still ends
in blocked compiler response without SQL/parameters/logicalPlan emission.

Trace FR-19/22, CONTRACT-007 and shared STP-056 B09/B12/B15/B16, US-056
AC7/AC10 prerequisites. SQL semantics, original parameter origin/domain mapping,
complete capability/obligation correspondence, native installation/authority/
execution/publication and formal refinement remain separate requirements. No
full workspace, installed Python/browser, native backend case or full profile is
promoted; all132 original cases remain required and shared goal active.


### Exact physical parameter domains — 2026-10-12

The compiler now distinguishes bounded physical shape from exact parameter-domain
correspondence. The latter reuses original ordinary compiler value semantics and
rejects unknown selected facets rather than ignoring them. Eighteen controls cover
empty/astral strings, NUL, canonical/noncanonical boolean, signed int8 boundaries,
overflow/fractional/unbounded integer, decimal precision/scale/exponent and unknown
string/integer/decimal facets. Shape can pass while domain checking refuses;
neither permits emission or actual native execution.

`evidence/security-physical-domains-20261012/receipt.json` retains the current
source archive/pins and fresh Rust1.90.0 release run:257 unit +69 security admission
+3 literal tests,329 passing total, zero failed/ignored. Original query/parameter
origin matching, complete capabilities/obligations, native codecs/placeholder/SQL
meaning/current authority and final publication remain required. No full workspace,
Python/browser, formal/native refinement or backend acceptance is claimed.
Trace FR-19/22, CONTRACT-007 and shared CONTRACT-063/STP-056 B09/B12/B13/B15,
US-056 AC7/AC10 prerequisites. Broader parameter domains remain explicit open
integration work, not a product language limit. All132 original cases remain
required; full shared implementation goal stays active.


### Original query execution-slot correspondence — 2026-10-12

The compiler now derives bounded scalar-value occurrences from its original
resolved query and compares exact physical execution slots under the explicitly
fully parameterized profile. Same-domain changed value, incorrect occurrence
path, omission and extra slot all refuse while shape/domain alone pass. Source
paths preserve join/on/filter/tuple/key occurrence identity; field-to-field values
are not invented parameter slots. Original SQL meaning/placeholder use and other
parameter sources remain open, so compiler emission continues to refuse.

`evidence/security-query-slots-20261012/receipt.json` retains current source pins,
archive and Rust1.90.0 release verification:257 unit +70 security admission +3
literal tests,330 passing total, zero failed/ignored. The initial compile caught
an extra dereference in the new borrowed-type comparison; it was corrected before
execution. Derived inventory refuses beyond1024 occurrences before accumulation.
No native SQL/codec/authority/lifetime/publication, formal refinement, full workspace
or installed Python/browser qualification is claimed. Trace FR-19/22,
CONTRACT-007 and shared CONTRACT-063/STP-056 B09/B12/B13/B15, US-056 AC7/AC10
prerequisites. No backend case is promoted; original132-case goal stays active.


### PostgreSQL execution placeholder coverage — 2026-10-12

The original query-slot component now has a separate PostgreSQL ordinal coverage
check using the existing workspace-pinned sqlparser0.63.0 tokenizer. Nine controls
cover repeat `$1`, quoted/comment/dollar-quoted fake references, unused slots,
undeclared/zero/noncanonical ordinals, question-mark strategy and lexical failure.
The compiler still emits no security SQL; placeholder coverage is not query/SQL
meaning, dialect registration or native execution admission.

Core's explicit sqlparser dependency edge was missing in the initial compile and
was added; the retained first log records that build error. Cargo.lock changes
only that Core dependency edge, with no version changes. Fresh offline locked
Rust1.90.0 release verification passes257 unit +70 security admission +3 literal
checks,330 total, zero failed/ignored. `evidence/security-postgres-placeholders-20261012/`
retains original source pins/archive and failure/current logs. Native/Python/browser,
full workspace, formal refinement and complete allocation guarantees remain open.
Trace FR-19/22, CONTRACT-007 and shared CONTRACT-063/STP-056 B09/B12/B13/B15,
US-056 AC7/AC10 prerequisites. No backend case is promoted; all132 original
cases remain required and full shared goal stays active.


### Execution slot context-source binding — 2026-10-12

Occurrence paths alone could coincide across same-shaped queries. Query-slot
origins now include an owner-derived context fingerprint over versioned framed
original inputs and resolved query values. Wrong fingerprint refuses alongside
wrong value/path/count. A real second compiler context with only an additional
source SQL space rejects the first context's retained slot metadata despite
identical logical value/type/path; both compiler responses remain blocked.

`evidence/security-slot-context-20261012/receipt.json` retains source pins/archive
and fresh offline locked Rust1.90.0 release verification:257 unit +71 security
admission +3 literal tests,331 total passing, zero failed/ignored. Original model
bytes are borrowed and resolved plan JSON is streamed into the hash. This is
source correspondence under SHA256 collision resistance, not issuer authentication
or authorization. No SQL meaning/native codec/source-lifetime/publication or
formal/native refinement, full workspace or Python/browser qualification is
claimed. Trace FR-19/22, CONTRACT-007, shared CONTRACT-063/STP-056 B09/B11/B12/
B13/B15, US-056 AC7/AC9/AC10 prerequisites. No original backend case is promoted;
all132 cases remain required and the full shared security goal stays active.


### Security pipeline workspace and ownership integration — 2026-10-12

After physical shape/domain/query-origin/placeholder/context changes, a fresh full
Rust workspace release run passes618 tests across46 groups, zero failed/ignored/
filtered. This includes native-host compilation of embedding/backend crates; it
does not execute installed Python bindings or actual WASM/browser artifacts.
`evidence/security-workspace-20261012/receipt.json` retains exact command,
Rust1.90.0 inputs, source archive/pins and original logs.

Strict module ownership now registers the five reviewed Core context APIs, private
security_physical_check source and workspace-pinned sqlparser0.63.0 dependency.
Only those reviewed entries change (24 additive map lines); checker implementation
and all other ownership entries remain unchanged. Cargo boundary checks and3
negative/positive boundary tests pass; strict11-package surface checking and all16
Bun boundary tests pass. Initial strict invocations lacked required WEFT_CARGO;
those configuration-error logs are retained. The intermediate map ordering was
corrected to existing lexical order before final strict validation.

This closes the current Rust source/dependency ownership integration gap, not
B-009's broader original duties, hosted CI, installed bindings, native security
lowering or any full backend case. Trace FR-19/22, CONTRACT-007/ADR-002 and shared
CONTRACT-063/STP-056 B09/B11/B12, US-056 AC7/AC9/AC10 prerequisites. Compiler
security emission remains blocked; all132 original cases remain required and full
shared security goal stays active.


### PostgreSQL placeholder review correction — 2026-10-12

Review found a P2 lexical correspondence defect: sqlparser0.63.0 could expose
parameters hidden by native operator-adjacent comments or continued escape
strings. It also accepted numeric dollar tags and Unicode identifier junk after
ordinals. A bounded token preflight now refuses these forms. Ordinary operators,
quoted identifiers, comments, standalone escape strings and ASCII/untagged dollar
strings retain positive controls. Unicode dollar tags and Unicode U& string
literals/custom UESCAPE remain explicitly unsupported by this component.

`evidence/security-placeholder-review-20261012/receipt.json` retains original and
corrected public-context probes, source pins/archive, targeted release logs and
independent PostgreSQL18.6 observations. All331 targeted tests pass (257 unit,
71 security admission,3 literals);11-package ownership and16 Bun boundary tests
pass. An initial boundary invocation omitted required WEFT_CARGO; the corrected
invocation and both logs are retained. Native evidence records50 exact lexical
cases under UTF8/standard_conforming_strings=on:35 prepared/executed cases and15
refusals. These native observations demonstrate finite scanner differences; they
do not qualify complete security SQL or application-query equivalence.

The public compiler stays blocked. Full workspace, hosted CI, installed binding
parity, security lowering, native authority and full backend acceptance are not
refreshed by this patch. Trace FR-19/22, CONTRACT-007 and shared CONTRACT-063/STP-056
B09/B12/B13/B15; no acceptance criterion or original backend case is promoted.


### Lowering obligation coverage design and formal analysis — 2026-10-12

CONTRACT-007 now specifies original owner-derived requirement coverage, complete
source references, distinct enforcement sites, bounded prerequisite DAG validation
and current independently checked settlement. It retains scan/action obligations
for zero-rule and field-free queries, ordered output/key identity, private-fact
completeness and source/authority/publication duties. Concrete source-token wire
encoding and native proof validators remain explicitly unimplemented.

The actual Z3 4.15.4 spike executes nine five-source/five-obligation checks:
six UNSAT violations and three SAT positive/negative witnesses. The negative
witnesses show why all-declared-enforced can omit authority coverage and why
accepted flags do not imply actual native enforcement.
`evidence/security-obligation-model-20261012/receipt.json` retains the exact
script, results, command and hashes. This formal analysis covers the conditional
finite design, not executable Rust refinement, native SQL or lifetime.

Next implement the owner source inventory and declaration coverage/DAG comparison
against actual compiler contexts, then the selected independent evidence validators
and original native admission runners. Trace FR-19–22, CONTRACT-007 and shared
CONTRACT-063/STP-056 B09/B11/B12/B15 and US-056 AC7/AC9/AC10 prerequisites.
No backend case is promoted, all132 cases remain required, and the full goal
stays active. Existing physical checkers and public blocked compiler behavior
remain unchanged.


### Obligation graph implementation and bounded oracle — 2026-10-12

The actual compiler context now exposes a pure bounded obligation-shape checker,
with charged declaration text and iterative prerequisite DAG validation. It
refuses ambiguity, unresolved/self/cyclic prerequisites and exhausted budgets.
The strict module map registers only the new public method; its initial ordering
mismatch was corrected before final ownership validation.

Fresh Rust1.90.0 offline locked release tests pass332 total:257 unit,72 security
admission and3 literal tests. The added original-context test exhausts all512
three-node directed graphs against an independent transitive-closure oracle
(25 acyclic positives,487 cyclic refusals), checks a4096-node chain and long cycle,
and crosses the exact32768-edge budget. Strict11-package ownership and16 Bun
boundary tests pass. The first test build had a borrow error in the duplicate
source mutation; fixed and retained as failed-build provenance.
`evidence/security-obligation-shape-20261012/receipt.json` retains sources/archive,
commands and original failure/final logs.

This implements declaration/DAG shape, not complete owner source coverage or
independently validated evidence settlement. Empty lists and opaque source IDs
are deliberately demonstrated as shape-only successes, with public security
compilation still blocked. No installed Python/WASM/browser, full workspace,
native policy or lifetime qualification is refreshed. Trace FR-19/22,
CONTRACT-007, shared CONTRACT-063/STP-056 B09/B11/B12/B15 and US-056 AC7/AC9/AC10
prerequisites. No original backend case is promoted; all132 remain required and
the full goal stays active.


### Context-bound owner event coverage — 2026-10-12

Actual source inspection found the existing private owner event issuer and
more detailed independent template/deployment matching. The new pure context
API reuses that issuer and preserves its old protocol. A separately versioned
exact-context frame supports returned declaration metadata and complete event
source comparison after bounded DAG validation. It does not introduce another
semantic issuer or admit a generic all-events obligation as security enforcement.

Fresh offline locked Rust1.90.0 release verification passes333 tests:257 unit,
73 security admission and3 literal tests. The added test removes every required
source individually, adds an unowned source, covers duplicate output aliases and
COUNT lineage, and rejects source SQL/binding/profile replay. Binding and profile
changes preserve identical underlying event IDs, so their refusal specifically
exercises exact-context framing. The first binding control omitted target digest
update and stopped at WFT-PIN; the corrected fixture reaches the actual checker.
Strict11-package ownership and16 Bun boundary tests pass.
`evidence/security-source-coverage-20261012/receipt.json` retains source pins,
archive, commands and original failed/final logs.

Complete typed occurrence/capability/template/deployment coverage, native evidence
validation and current authority/publication admission remain required. No full
workspace, installed Python/WASM/browser or native qualification is refreshed.
Trace FR-19/22, CONTRACT-007, shared CONTRACT-063/STP-056 B09/B11/B12/B15 and
US-056 AC7/AC9/AC10 prerequisites. No original backend case is promoted; all132
remain required and the full security goal stays active.


### Registry-bound candidate correspondence integration — 2026-10-12

A combined pure registry API now connects original declaration selection and
private coherent semantic coverage to typed owner/rule custody and the existing
physical-domain/query-slot/result/event-source checks. It does not dispatch
backend lowering or emit an executable artifact. Candidate metadata requires
explicit comparison selection; Supported metadata remains unqualified.

Fresh Rust1.90.0 offline locked release verification passes334 tests:257 unit,
74 security admission and3 literal tests. The original-context integration
fixture has a panic-on-lower backend and verifies eight candidate mutations,
six selected registration mutations, foreign registry identity and Candidate/
Supported metadata controls. `SELECT 1` deliberately passes physical declaration
shape for a field query, demonstrating absent SQL equivalence rather than
claiming enforcement. Public compilation remains blocked without SQL/parameters/
plan. Strict11-package ownership and16 Bun boundary tests pass.

The first integration test's Supported manifest omitted required declared
evidence metadata and correctly failed registration; the fixture was corrected
with explicitly unverified evidence strings.
`evidence/security-candidate-declaration-20261012/receipt.json` retains original
failed/final logs, sources/archive and commands. No independent proof or native
case is replaced by those strings.

Next connect independently authored template/deployment matching and selected
binding meaning with actual native admission. Complete SQL/query/codec
correspondence, authority lifetime and publication remain required. No full
workspace, installed Python/WASM/browser or native profile is refreshed. Trace
S04, FR-19/22, CONTRACT-007, shared CONTRACT-063/STP-056 B09/B11/B12/B15 and
US-056 AC7/AC9/AC10 prerequisites. No original backend case is promoted; all132
remain required and the full goal stays active.


### Independent matched-instance candidate bridge — 2026-10-12

The private instance correspondence now compares returned lowering obligations
with retained original registry declarations after the existing independent
matching gate. Its original owner context supplies current frames; selected-only
deployment duties preserve empty semantic-source projections without invented
paths. The composed check requires exact capability selection, including zero-edge
selection, plus physical-domain/query-slot/result correspondence. Legacy unscoped
shape behavior stays unchanged and no public admission/dispatch path is opened.

Fresh offline locked Rust1.90.0 release tests pass335 total:258 unit,74 security
admission and3 literal tests. The independent-template fixture exercises every
obligation omission, ten isolated metadata mutations, invented selected-source
and duplicate-ID controls, reordered metadata and six combined physical/capability
corruptions. Strict11-package ownership and16 Bun boundary tests pass.
`evidence/security-matched-candidate-20261012/receipt.json` retains exact sources,
archive and commands/logs.

This proves finite tested conditional declaration correspondence, not authenticated
template completeness, independent native case execution, SQL/query equivalence
or original authority/publication lifetime. No general formal Rust refinement,
full workspace or Python/WASM/browser qualification is refreshed. Actual deployment
case duties and native gates remain mandatory before lowering/emission. Trace
CONTRACT-006/007, S04, FR-19/22, shared CONTRACT-063/STP-056 B09/B11/B12/B15 and
US-056 AC7/AC9/AC10 prerequisites. No original backend case is promoted; all132
remain required and the full goal stays active.


### Current security workspace integration — 2026-10-12

After declaration DAG, exact-context event coverage, registry candidate checks
and independently matched projection changes, a fresh full native Rust workspace
run passes622 tests across46 groups, zero failed/ignored/filtered.
`evidence/security-current-workspace-20261012/receipt.json` retains original
command/log, Rust1.90.0 source attribution and post-run archive at0f06bc5.
This is workspace integration, not a held source interval, hermetic binary
provenance, installed Python/WASM/browser or native security qualification.

Current shared case inventory still has132 required cases and106 null commands:
16 pg-raw cases and30 each Truss/Delta raw/Ashlar. The integrated compiler
components do not implement those missing original runners. No backend case is
promoted; the full goal and every required native/authority/publication duty
remain active. Trace S04/S11/S12, CONTRACT-006/007 and shared STP-056/057.


### PostgreSQL candidate physical prerequisite composition — 2026-10-12

The explicit registry PostgreSQL candidate check now requires the selected immutable target to declare exact postgresql/18.6 with UTF8 client/server encoding, standard_conforming_strings=on and backslash_quote=safe_encoding. It composes existing candidate declaration correspondence with execution ordinal coverage, so a caller cannot omit that physical prerequisite by invoking the combined entrypoint. Generic declaration checks retain their existing dialect-neutral scope. Neither method admits SQL/query/result equivalence or calls backend lowering.

[Evidence](evidence/security-pg-candidate-20261012/receipt.json) retains335 passing targeted Rust1.90.0 tests, nine original native PostgreSQL18.6 parameter-inventory controls, exact sources,11-package ownership and16 boundary controls, and Astra Ultra review. Initial missing session-setting premises were fixed with eight omission/mismatch controls. Native PostgreSQL accepts $01 while the selected declaration profile requires canonical ordinals. All132 required cases remain; no backend case is promoted. Complete physical/result/dependency/native-source coverage and authenticated authority/publication remain open.
### Truss PG18 host composition — 2026-10-12

The Truss consumer has authorized building its backend here against the
DocumentDrivenDX pgserver PostgreSQL 18.6 runtime. The normal Python extension
feature `truss-postgresql-pg18` exports
`compile_truss_postgresql_json(request_json, composition_json)`. Rust exposes the
same implementation through `weft_postgresql::pg18::compile_json`.

The host supplies original data-only storage declarations separately from the
SQL/model request. The bridge admits their bytes and pins through the existing
original backend, with statically selected scalar procedures. It does not select
fixture presets, start subprocesses, connect to a database, or evaluate authority.
It requires backend `truss.postgresql.original` version `0.2.0`, target
`truss-pg18.6-live/0.1`, and composition version
`weft-truss-pg18-composition/0.1.0`. Historical PG17 profiles remain separate;
the new manifest carries no inherited native evidence. Candidate opt-in and
emitted host obligations remain required. Native row-tree procedures are not
installed in this bridge. The target explicitly requires
`search_path=pg_catalog,pg_temp` during prerequisites and query execution, so
caller schemas cannot substitute unqualified operators.

This compiler entry is a prerequisite for Truss's public compiled-query provider.
That provider still needs real accepted model/storage declarations, independently
safe logical projections over its tagged cells, native parameter binding,
execution of emitted obligations and exact result decoding. A scalar filter
transport check is not native PG18 execution evidence or closure of the security
compiler work. The next usable workflow is public registration and mutation,
followed by Weft filter/count compilation and authorized execution in the caller's
psycopg transaction; connections and transaction settlement remain host-owned.

The connected Truss runtime adapter now has a separate binding and normal
`compile_truss_runtime_json` entry point. It admits complete resolved member
identities against original UMF source and actual record/property mappings, then
uses existing Weft IR, parameter slots and result metadata. The adapter lowers
equality filters/joins, scalar/complete-record projections, counts, ordering and
limits to a statically named Truss source function. Optional projections explicitly
assess `value.nativeNull`; the common emission validator remains intact.
Truss owns source validation, ordinary-person authority, one-statement data
snapshots, current context checks and exact decoding. Real installed consumer
workflows and the .25→.26 migration have passed on pgserver-pg18==18.6.1/PG18.6.
The evidence is scoped to string/boolean fields and integer COUNT; it does not
close wider numeric/relationship/page-key support or security-compiler activation.


Post-merge validation preserves upstream8129b19's ordinary Truss PostgreSQL/runtime additions. [Merged workspace evidence](evidence/security-pg-candidate-20261012/merged-workspace/receipt.json) records624 passing tests across46 groups, zero failed/ignored/filtered. The strict inventory initially exposed seven upstream source/API entries missing from main's map. Astra independently checked those exact files/declarations against upstream before the explicit seven-entry repair; generic whole-inventory regeneration was rejected and not executed. Ownership now passes11 packages and16 controls. This does not qualify ordinary runtime additions as security admission or refresh installed bindings.


### Shared logical fact projection consistency — 2026-10-12

[Retained sources and controls](evidence/security-fact-consistency-20261012/receipt.json) align general and scoped Weft record-fact simulation with UMF's known keyed projection rule. General simulation now refuses conflicting subject/inventory/resource values; scoped simulation permits consistent partial projections instead of requiring identical entire field maps. Missing policy/output fields still refuse on their own selected projection, and null/absence remain distinct. Borrowed normalized values avoid additional payload retention; comparisons retain the existing conservative budget.

Astra independently verified scoped join/masked combinations and the common fold after correcting one obsolete partial-subject rejection assertion. Native/compiler admission remains closed: no source/refinement/native-snapshot proof or required backend acceptance case is supplied by this logical change. All132 required cases remain in scope.

Full workspace validation passes626 tests across46 groups, zero failed/ignored/filtered. Strict ownership passes11 packages and the independent Cargo manifest boundary check passes11 modules/12 edges. No inventory regeneration was required; the new fold is private.

### Truss exact token projections — 2026-10-12

Backend0.4.4 extends the public PG18 integration with projection-only timestamp
and binary descriptors under compile/IR/backend interfaces0.2.1. Comparable
families and SQL dialect0.2.0 stay unchanged; old schemas/backends remain closed.
The existing backend traits and pure compiler dispatch are reused. Generic
backend codecs explicitly refuse these values rather than treat them as strings.

The installed Truss mixed scalar workflow and preview0.4 pass on the merged
Rust/Python build; original backend0.4.3 consumers remain usable. Astra Ultra
reviewed the plan, implementation and merge. [Receipt](evidence/truss-exact-projections-20261012.json)
records scoped native results and artifact hashes. Timestamp arithmetic, binary
comparison, new browser packages and protected security activation are not claimed.


### Candidate graph fact consistency — 2026-10-12

[Retained regression and implementation evidence](evidence/security-graph-fact-consistency-20261012/receipt.json) closes the draft graph simulator's known-field agreement gap relative to the previously corrected record simulator. The public prechange test demonstrates acceptance of conflicting same-identity subject/resource projections. The fix checks all supplied stored projections before interpretation, including unused raw and record-backed graph witnesses, while preserving context separation and missing-field refusal on the actual evaluated projection.

162 core security tests, all74 admission tests and11 workspace boundary checks pass. The separately bounded borrowed preflight supplies no native source/authentication, physical lowering, ready profile, null/absence extension or backend case admission. All132 original required cases remain in scope.


### Source-bound security mapping host transport — 2026-10-13

[Retained implementation and original transport evidence](evidence/security-mapping-transport-20261013/receipt.json) exposes `prepare_security_mapping_json` through native Python, WASM and a bounded CLI. It consumes original source strings and SQL, reuses the owner query profile and SQL-derived scan/operator lineage, and returns the existing mapping handoff with separate action/dependency closures. Caller IR/action annotations are not accepted. `securityReady:false` remains unconditional; no native permission or security SQL is issued.

Astra found repeated parameter text could reach per-occurrence resolver copies before the output-size check. Owner-AST preflight now applies a separate selected-parameter payload ledger before substitution; the exact-boundary and64KiB×300 controls pass. Profile whitespace controls intentionally prepare new hashed source inputs, demonstrating that transport custody is not issuer authentication.

Concurrent main's complete-entity and relationship changes were merged before final builds.357 affected core/admission/application tests, the CLI byte-boundary test and11 package-boundary checks pass. The rebuilt original imported Python cdylib/CLI and actual Chromium153 WASM pass16 controls with all response hashes agreeing. A valid authored relationship model still prepares an ordinary field query; relationship shorthand and the actual new complete-entity profile refuse at their declared boundaries. Astra independently ran6 core tests and18 CLI controls before integration, then verified merged source/evidence custody and final native/browser correspondence. Retained commands, sources, pins, handoff and initial corrected attempts qualify this component. No installed wheel/native SQL/roles/effects/source-cut/issuer/guard/release or whole-heap/CPU/source-refinement proof is claimed.

US-009-AC1/AC2 and original pg-raw.B08 receive prerequisite evidence only. Next, backend-native mapping must consume this original handoff under authenticated installed profile/caller custody, establish exact SQL/parameter/effect correspondence and enforce the complete guard/settlement/release protocol. All132 original required security cases remain in scope; no case is promoted by this transport.

### Conditional predicate spike — 2026-10-13

The experimental `security_predicate_spike` module consumes the original admitted immutable policy plan, catalog and matching native metadata. It emits bounded string/boolean rule expressions with nominal ordered keys and K3 existential truth, including unknown witnesses. It is not registered security compilation: `securityReady:false` remains unconditional. Complete originally validated facts, declared witness keys, truthful current mappings, authentic helpers and `pg_catalog,pg_temp` search path are explicit premises. Context and other scalar families refuse.

[Independent Astra review](evidence/security-predicate-spike-20261013/review.json) and [11 independent controls](evidence/security-predicate-spike-20261013/results.json) qualify this diagnostic subset. The example builds with Rust1.90. A Truss PostgreSQL18.6 diagnostic populated 12 rows through the ordinary writer and produced six expected membership truths; its separate runtime changes remain under validation and are not a backend acceptance promotion. Full crate tests remain blocked by existing `ExactScalar` test fixture matches. No authenticated caller, RLS, complete fact admission, protected query or formal implementation equivalence is claimed. All132 required cases remain in scope.

### Additive Truss0.37 ordinary-query compatibility — 2026-10-13

The import-load backend profile now accepts exact Truss0.37 alongside its original0.35/0.36 releases; other backend selections remain unchanged. The original Python module rebuilds with Rust1.90 and native pending/committed read-only import-load queries pass PostgreSQL18.6 (8.133s), retained in [native evidence](evidence/runtime037-compatibility-20261013/native-query.log). This preserves existing ordinary-query support. Nonempty signed64 facets remain refused by native query_objects; no signed64 query qualification, security-ready profile or acceptance case is promoted.

### Conditional K3 lowering lemmas — 2026-10-13

[Executable Z3 proof](evidence/predicate-k3-20261013/proof.py) and [results](evidence/predicate-k3-20261013/results.json) retain12 UNSAT discrepancy checks with satisfiable premises: K3 AND/OR/NOT and existential witness cardinalities0–8. A SAT weakened-EXISTS counterexample shows why SQL `EXISTS WHERE condition` loses an unknown witness. These are abstract finite truth-algebra lemmas, not parser/SQL implementation, source completeness, native authentication or unbounded existential proofs. The diagnostic lowering's actual native aggregate requires separate correspondence evidence. All required backend acceptance cases remain open at their previous statuses.

### Closed predicate diagnostic transport — 2026-10-13

The separate example now returns structured refusal instead of panicking on malformed input. Its top-level envelope admits only request/assessment and paired optional cut/evaluationRequest, preserves original owner parsing, bounds UTF-8 ingress and final serialized output to16M, and refuses transport I/O errors. [11 malformed controls](evidence/predicate-transport-20261013/refusals.json) cover empty/malformed/scalar/array input, missing source members, foreign/duplicate envelope fields, unpaired simulation data, invalid UTF-8 and excess bytes. [Original native/simulator regression](evidence/predicate-transport-20261013/native-regression.log) passes PostgreSQL18.6 (8.859s). This is diagnostic source transport, not compiler/principal/issuer authentication, complete heap accounting, protected query admission or a ready backend. The required backend corpus stays in scope.

### Whole-policy diagnostic eligibility in progress — 2026-10-13

The experimental predicate packet0.2 adds truth-column identities and selected original target/action group decision expressions. Composition mirrors original empty-disclosure decisions: any selected unknown is indeterminate; a true permit is required; every require must be true and every forbid false; otherwise deny. Missing groups cannot authorize. This is not disclosure/mask composition, caller authentication, source completeness, original-use authorization or protected-query admission. Group membership consumes the original work ledger and a separate1MiB copied-identity/action byte budget before allocation; output/SQL bounds remain.

Initial native actual-population test passes: six SQL eligibility decisions agree with original simulator retained/filtered resources. Final native truth-matrix replay and Astra source review are pending before direct-main integration. All132 required backend cases remain in scope and unchanged; this does not promote truss.B01 because ordinary native principal selection is still absent.

The final native selected-group replay passes27 permit/require/forbid vectors in original and reversed rule order, nine reader/require vectors and six original-population simulator decisions. Truth-vector SQL supplies abstract columns directly; native fact derivation remains separately qualified. Explicit `missingEligibilityGroup:unsupported_refuse` closes the absent-group interpretation gap. Astra confirms grouping budgets precede retention and unknown-first composition matches original empty-output compose. No full authorizer, mask conflict handling, original-use gate or backend acceptance promotion is claimed.

Final independent Astra review has no remaining actionable findings:13 controls/141 emitted CASE evaluations, exact/over8192 work and1MiB metadata bounds. Its decision evaluations use in-memory SQLite and original retained composition vectors, not a PostgreSQL oracle; the separate six-test PostgreSQL18.6 suite passes27.386s. [Source-pinned review](evidence/whole-policy-eligibility-20261013/review.json) qualifies this selected-group empty-disclosure diagnostic only. Direct-main integration does not activate public security compilation or promote a backend case.

### Original/withheld disclosure-state diagnostic in progress — 2026-10-13

Diagnostic0.3 computes selected original ontology field states from original permit rules. Withheld dominates original; active original disclosure selects original; a protected field with no active disclosure is indeterminate; unprotected fields default original. State expressions are diagnostics, not value projection or a complete output decision. Transformations explicitly refuse in this increment and remain required by the full design, including exact literal equivalence and conflict handling. Field identity copies charge the shared1MiB metadata ledger before allocation and disclosure incidences charge work.

The separate transport now retains original simulator disclosure dispositions. Native initial control passes8.906s: salary withheld and unprotected identity original match original composition on six actual native population pairs; denied resources are excluded. [Native evidence](evidence/disclosure-state-20261013/native.log) is component-only. Final bounded-source regression and Astra review are pending before main integration. No authenticated source/caller, SQL masked projection, transforms, original-use gate, complete result contract or backend acceptance promotion is claimed.

Review corrections:field-state SQL is now row-permit gated; denied/indeterminate rows return `not-applicable`, not a disclosure. A permitted protected field whose state is indeterminate must make projection refuse. Source lookup includes original association as well as entity targets with exact borrowed reference components. The gated six-test PostgreSQL18.6 suite passes27.052s. Every selected rule-field inspection now consumes work even for empty disclosure or non-permit rules, avoiding an uncharged product loop; final source replay/review remains pending.

Final native original/withheld precedence replay passes9.627s, covering18 emitted field-state vectors across both original rule orders and original simulator disclosure correspondence. The scoped Z3 two-permit protected-field lemma yields UNSAT for abstract gated-state discrepancy and SAT for an ungated disclosure counterexample; it does not prove implementation/transform/value/null/absence/authentication. Final source review is still running before direct-main integration.

Independent Astra corpus passes117 controls/164 emitted-state evaluations, including original retained disposition vectors in both orders, active/unknown permits, association targets, transform refusal and exact/over8192-work/1MiB metadata. These state evaluations use in-memory SQLite, not independent PostgreSQL; native checks remain separately retained. Final build and native work replay pass. Scope remains row-gated original/withheld states, with indeterminate field requiring projection refusal; transforms/value projection/authenticated execution are still required. Direct-main integration promotes no backend cases.

### Original-composer transformation states in progress — 2026-10-13

Diagnostic0.4 adds transformed/conflict states and row-gated transformation choice slots with retained original disposition metadata. Pairwise equivalence/conflict delegates to original `security_composition::compose` and its exact literal/refinement semantics; backend code does not compare raw numeric strings or introduce another literal interpreter. Withheld wins over conflict; nonpermit rows are not-applicable; choices exist only for transformed state. Indeterminate/conflict fields require projection refusal. Pair comparisons consume work, retained serialized transformation metadata consumes the1MiB metadata ledger, and expression/total SQL/final output limits remain explicit. Source-bounded serialization scratch and owner composition allocation are not whole-heap/CPU proofs.

Initial PostgreSQL18.6 control passes9.046s with54 emitted state/choice vectors covering original numeric0/-0 equivalence,1/2conflict, withheld precedence and unknown/nonpermit gating, alongside original fact/simulator correspondence. [Native evidence](evidence/transformation-state-20261013/native.log) qualifies transformation state selection, not SQL value projection, output-domain execution, masking completeness, authenticated principal/source/current authority or backend activation. Review/final-source replay remain pending before direct-main integration; all132 required cases keep their full scope/status.

The six-test native transformation regression suite passes26.791s. The final source separately rebuilds with explicit conflict-field projection-refusal metadata. A conditional Z3 three-transform normalized-identity fold yields UNSAT discrepancy and a SAT withheld-over-conflict witness; original tuple normalization remains supplied by the owner composer, not established by this abstract lemma. Final-source native replay and independent source review remain pending before direct-main integration.

Final independent Astra run passes875 renderer controls/1,026 state-choice evaluations:all432 original disposition vectors in both orders, numeric0/-0 and100/1e2 equivalence, conflicts, withheld/choice gates and exact/over8192-work/1MiB transformed metadata. Its expression evaluation uses SQLite with the typed-NULL spelling translated, not an independent PostgreSQL refinement oracle; native54 transform vectors and the separate six-test suite/final-source replay retain their narrower native evidence. No actionable source blocker remains. Direct-main integration keeps value projection, output-domain execution, authenticated source/principal/current authority and the complete132-case backend acceptance scope open.

### Diagnostic cell projection in progress — 2026-10-13

Diagnostic0.5 adds per-field tagged cell expressions and exact ordinal parameterTypes. Original native JSON-home scalars are converted to original CoreLiteral wrappers; withheld and absent omit value, explicit null remains a value. Transformed literal payloads are typed JSONB parameters, never SQL text interpolation;1MiB cumulative serialized parameter bytes/count1024 are checked before retained cloning. Transformation metadata/choice remains original-composer derived. Missing required/malformed cells, conflict/indeterminate and nonapplicable states emit refusal/nonapplicability markers; a future admitted host must reject/filter appropriately before any release. This diagnostic has no registered result contract or execution permit.

Initial native PostgreSQL18.6 control passes8.935s:actual original identity projection, salary withholding with no value member, and parameterized transformed integer0/1 lexical payloads. [Evidence](evidence/cell-projection-20261013/native.log) does not yet qualify positive absent/null model fixtures, all scalar families/output domains, native query-original operations, complete result batch refusal, caller/source/issuer/current authority or protected publication. Review and full final regressions remain pending before direct-main integration. All132 required backend cases retain their original scope/status.

Final8 native controls pass38.851s with original registered nullable/binary models and ordinary writer population. Explicit null remainsvalue:null, absence has no value, and canonicalbase64AQI=becomesbinaryHex0102 using native decode/encode. The original nullable salary aggregate/join query remainsWFT-TYPE-refused; identity SQL is a separately admitted projection fixture. No nullable query-operator support is inferred. The incorrect base64-as-hex source was caught and corrected before integration.

Astra21 renders/211 cell-branch evaluations pass, covering null/absence/refusal omission, transformed null, mixed parameter ordinals and exact/excess budgets. These independent branch checks use translated SQLite JSON operations with native-validity premises, not independent PostgreSQL validation; native evidence is separate. This diagnostic output remains unregistered and cannot authorize protected execution or release. No required backend case is promoted.

### Result outcome-class correspondence in progress — 2026-10-13

Original result declaration checking requires one outcome per normalized transformation class plus complete contributing rule provenance. Diagnostic0.5 retained separate per-rule outcome IDs, so equivalent constants could not satisfy that later contract. Diagnostic0.6 now derives canonical classes through the original composer pair semantics, emits one outcome ID for equivalent transforms, and retains rule IDs/per-rule slots/dispositions. Actual chosen literal spelling remains parameterized and preserved; no new normalized-literal interpreter is introduced. Added rule ID retention is charged before cloning in the1MiB metadata ledger.

[Native control](evidence/outcome-classes-20261013/native.log) passes10.403s:0/-0 sharetransform0 with exact selected lexical values;1/2 retaintransform0/transform1, alongside prior state/choice/disclosure/original fact checks. This fixes a prerequisite for closed result admission; it does not yet construct/check the full original context-bound result contract, validate complete output selection/batch, authenticate source/principal or permit execution/release. Review/full regressions remain pending before direct-main integration. All132 required backend cases remain unchanged.

Final8 native regressions pass40.314s. Independent Astra22-render/1,195 emitted-cell evaluations cover interleaved classes/reversed orders/trivalent truths, output-field identity, null/string classes, complete per-rule provenance and exact metadata limits. Those independent cell evaluations are separately qualified from native PostgreSQL; they do not test complete result-contract admission. Conditional three-normalized-ID proof yields UNSAT class discrepancy and SAT per-rule-class counterexample. Direct-main integration remains a result-contract prerequisite only, with no readiness or backend case promotion.

### Original-source result-cell inspection in progress — 2026-10-13

`inspect_security_mapping_result_cells` reconstructs the original closed4M mapping request, model, policy, query profile and resolved SQL before constructing the existing private owner context and delegating typed contract, exact retained bytes/hash and batch to `check_result_cells`. It returns pure correspondence only; no context constructor visibility, backend registration, physical plan, selection or release authority is added. Existing source transport remains unchanged and its six core tests pass.

The separate closed diagnostic resultProbe builds a single-column declaration from original source classification and its own generated canonical transform classes/provenance; owner checking rejects source/query/column/declaration/cell drift. Initial position0 was correctly refused; corrected original one-based position passes. Native positives9.354s and expanded negatives9.233s accept exact original/equivalent transformed cells and reject changed values, unknown outcomes, extra members, withheld values, arity and column drift with empty stdout. Astra52 independent controls cover exact bytes/hash, typed declaration/provenance, lexical classes and4096/4097 row bounds. These are pure correspondence controls, not native selection/issuer/snapshot/authentication or protected release. Full native regression remains pending before main integration; all132 backend cases remain in scope without promotion.

Final8 native regressions pass38.702s; all6 core mapping transport tests pass. Source-pinned Astra review has no actionable findings and52 independent controls pass. This directly integrated inspector checks original-source-bound declarations/cells only; complete rule selection, physical lineage, issuer/principal/current authority and protected release remain required with no backend-case promotion.

Merged-source validation preserves concurrent runtime0.38 durable feed receipt changes. The rebuilt diagnostic passes all8 native regressions in40.718s against the merged runtime; Weft declared API inventory and all16 boundary controls pass. Retained evidence: `evidence/result-inspection-20261013/merged-runtime038-native.log`. Source mapping uses the separately qualified original0.37 mapping extension; this does not qualify registered0.38 protected execution. No required backend case is promoted.

### Original-owner scoped-fact selection — 2026-10-13

The diagnostic now invokes the existing original owner's conditional fact-selection checker after original source/query reconstruction. Captured native facts and actual scan occurrence IDs drive per-rule evaluation without caller truth overrides. Actual native withheld and original resourceId cells pass for permitted Staff/Project memberships; well-formed cells on denied combinations, forged original values and inconsistent scoped facts refuse without stdout. All8 native regressions pass40.117s on merged runtime0.38/PostgreSQL18.6; original owner scoped-fact test1 and boundary controls16 pass. Retained source pins/logs: `evidence/fact-selection-20261013/receipt.json`. Population completeness/trust/generation are caller assertions; native SQL lineage, complete enumeration, authenticated caller/source and guarded release remain open. No required backend case is promoted; all132 remain required.

Astra Ultra found no actionable blocker in this conditional correspondence increment:41 independent current-binary controls pass,13 original owner/source/schema files are pinned, all4 native receipt pins match, and retained native8/owner1/boundary16/transport6 logs were inspected. Native authentication, complete result enumeration and guarded release remain required and unproven.

### Simultaneous multi-column native batch correspondence — 2026-10-13

The closed diagnostic fixture now builds1–256 ordered result columns with conservative16M source-expansion reservation before repeated declaration copying, preserving legacy single-column inspection. One native prepared statement projects resourceId and protected salary together through original renderer predicates and permit filtering. Original owner batch/declaration/fact checks admit actual cells; swapped cells, reordered declarations, missing cells and a forged original value in a later row refuse without stdout. All8 native regressions pass42.528s; boundary controls16 pass. Exact SQL/cells/source and binary hashes are retained in `evidence/multicolumn-native-20261013/receipt.json`, explicitly qualifying final fixture overwrite scope. Production caller/issuer authentication, original SQL lineage, complete enumeration, writer closure and guarded release remain required. No acceptance case promotion; all132 remain required.

All41 previously retained single-column fact/cell controls were replayed against the new multi-column diagnostic binary and pass; this replay is parent-run regression evidence, separately recorded from current independent review.

Astra Ultra found no actionable blocker:34 new multi-column controls,41 independently rerun legacy controls and9 retained packet/source checks pass. Exact256-column and16M source-copy reservation positives and excess refusals are retained. Actual native qualification remains the two-column finite fixture; authenticated issuer/caller, production enumeration, compiler lineage and guarded release remain unproven.


### Original ontology subject selection — 2026-10-13

The source-bound Weft inspector reconstructs the original catalog, ontology, policy, SQL and admitted query profile, then checks the selected source SHA/Record/Key against the original ontology Subject. A closed source-only diagnostic returns `securityReady:false`; no native admission is produced. Truss requires this synchronous inspector before native observation or original UMF key encoding. Deferred async/generator functions are refused before invocation, including across realms; ordinary rejected Promise returns are observed before refusal. Arbitrary configured callback effects, hidden asynchronous wrappers and modified Promise constructors/species remain trusted-code premises. Original helper authenticity, actual subject existence/current binding, issuer authority and protected release remain unqualified. Retained evidence: `evidence/ontology-subject-selection-20261013/receipt.json`. Seven core transport tests, sixteen module controls, twenty-one host tests, thirteen native subject groups and eight predicate regressions pass; independent actual-binary review passes thirty-nine controls with every refusal producing empty stdout. Owner-only selection callbacks are test doubles; native fixtures use the original Weft inspector. No required backend case is promoted; all132 remain required.


### Subject selection through original consumer bindings — 2026-10-13

The original source inspector now has a closed JSON response transport through runtime, Python and WASM. Its successful response is `inspected` with `securityReady:false`; it returns no native SQL, subject credential or release authority. Actual rebuilt Python extension14 controls pass. Actual Chromium WASM14 responses match Python byte-for-byte, including stale source/profile, foreign Record/Key, duplicate/unknown selection members, malformed requests, size and invalid SQL refusals. Core transport7 and ownership16 controls pass. Evidence/source and artifact hashes are retained in `evidence/subject-selection-consumers-20261013/receipt.json`. This supplies an original-owner consumer entry point; it does not authenticate configured code/helpers or current subject/issuer/native authority. No required backend case promotion; all132 remain required.


Astra Ultra independently passes22 actual loaded Python-extension controls, including exact/over selection16,384-byte and original request4,000,000-byte limits. It verifies all14 retained Chromium response strings and artifact hashes against disk, separately from a browser rerun. No actionable blocker remains in the pure source-inspection transport scope. Source-pinned review and independent controls are retained alongside the consumer evidence.

### Shared exact timestamp equality — 2026-10-14

Compile/IR/backend026 explicitly select committed UMF exact-instant1.0 without changing generic timestamp/facets/Key meaning. Backend0.4.29 binds native59, retains old28/58, dispatches exact equality by family, validates original operands before emission and preserves typed slot/occurrence custody. Basic public Truss07/08 workflows cover precise offset equality, optional values, joins/pages/read-only transactions; matching timestamp/binary/populated upgrade checks pass5/48.058s. Workspace Rust checking, one targeted exact-instant test and11-package boundaries pass. Astra source review is clear after the three Truss version/upgrade-custody fixes. Weft24 release uses the existing five-platform workflow; Ashlar admission remains unavailable.

### Registration-owned Truss rebinding — 2026-10-14

Weft25 adds backend0.4.30/native60 through a separate selection flag, preserving backend29/native59 and inheriting exact timestamp/scalar/container/entity/page semantics. Truss24's seven basic PG18 cases pass, including UMF0.7/0.8 scalar and complete-entity queries after registration, and populated native59 upgrade. Workspace Rust checking and11-package boundaries pass. Astra Ultra approves the mapping. Publication uses the existing five-platform workflow; broader Truss rebinding and Ashlar admission remain separate.

### B-008 compiler-owned lowering preparation — 2026-10-13

A pure exact PostgreSQL18.6 registry dispatch now checks original owner source/profile/semantic selection before one lowering callback and validates the entire returned declaration before retaining it. Immutable checked snapshots retain exact manifest, plan and result-contract bytes/hashes and original context fingerprint. The original public compiler still refuses activation until native mapping/lineage, authenticated caller/source custody and guarded release close. The security chat owns the Truss native admission/release path; Weft owns compiler binding/lineage/dispatch. No existing dirty checkout was reset or overwritten.

The expanded adversarial boundary test passes, and all74 original security admission tests pass with no filtered/ignored cases. Tests distinguish zero-call initial refusals, one-call valid preparation and whole-result refusal for changed slots, result name, capabilities or omitted obligations. These are compiler correspondence tests, not native enforcement or B-008 completion. Full original132-case scope remains open; native typed graph correspondence and public activation remain the next integration gates.


### B-008 reified graph compiler/native handoff — 2026-10-13

The graph compiler now consumes original separate logical and physical models, actual captured catalog inventory and explicit typed edge-path correspondence. It preserves logical keys and verifies native property/key/relationship declarations, emitting all-node endpoint checks. The original resolved direct query determines SQL/output order and the owner validates the typed disclosure result contract. A separate original producer/result inspector verifies exact source/output bytes and the whole cell batch; no typed declaration can be substituted by deserializing arbitrary JSON. Native Truss composition is owned by the coordinating security chat, preserving its source/caller/role/fact/revocation cuts.

Graph tests2 pass; actual producer controls16 and original result controls8 pass. Original core269/admission74 regressions pass. The initial prototype wrongly equated authored `r1` and native counter1; they are explicitly separate in the final carrier. The initial result example attempted to deserialize a deliberately non-deserializable contract; the final path retains the original owner-built typed `GraphQueryCandidate` instead. Neither attempt qualified native/public support. Source-only candidate/nativeAdmission required and all132 original obligations remain open. Native execution and original semantic-bag/result-source parity are the next gates; no smaller diagnostic scope closes B-008.


### B-008 original result-refusal diagnostic boundary — 2026-10-13

The original cell checker produces bounded `result`-phase refusals, but the shared callback sanitizer omitted that phase and replaced those refusals with a generic backend-version error. The sanitizer now preserves valid original result diagnostics; code, severity, message and SQL-span bounds still apply. Regression controls cover both the compiler factory and the original mapping-context callback, and existing malformed callback controls remain active. All75 security admission tests pass with no failures or ignored cases; [receipt](evidence/security-result-diagnostics-20261013/receipt.json) retains the exact raw-log hash and compressed log. This changes refusal reporting only, supplies no native admission or activation, and leaves B-008 and all132 original shared/native cases open.


### B-008 permanent graph-query and complete-batch regressions — 2026-10-13

The PostgreSQL integration suite now retains original-owner graph candidate controls for three output shapes, including repeated fields and changed order; six unsupported relational operations; a substituted native schema context; empty and populated exact result batches; and seven corrupted batches. The later-row corruption control proves the owner cell checker refuses the complete batch when a withheld cell carries a value. All5 graph integration tests pass, with no ignored cases. [Receipt](evidence/security-graph-regressions-20261013/receipt.json) retains the command, raw-log hash and compressed evidence. These are source/query/result correspondence regressions, not authenticated native enforcement or backend case promotion. B-008 and all132 original shared/native cases remain open.


### B-008 original parameter provenance for guarded lowering — 2026-10-13

The existing physical declaration only accepts original query values, while actual graph lowering also needs policy operands and an authenticated native subject. Core now issues a separate private-constructor borrowed parameter-source inventory from the original owner context. It keeps query occurrence values, condition/transformation constants, unvalued context operands and the original logical subject key distinct. It preserves explicit null, numeric token spelling and duplicate SQL occurrences. Another owner context cannot substitute equal bytes/digests. Existing original-query slot enforcement, registration tuples and public activation remain unchanged.

[Evidence](evidence/security-parameter-sources-20261013/receipt.json) covers core/admission regressions, six source-inventory controls, budget exhaustion and a compile-fail privacy check. The module ownership inventory is reconciled for this new source and the previously integrated graph/mapping owner APIs; actual eleven-package checks and16 boundary controls pass. No native case is promoted. Next, graph lowering must bind physical policy/context/native ordinals to these actual sources and consume Truss's registered source/subject issuers; full guarded dispatch and the132-case native/lifecycle corpus remain required.


### B-008 bind graph policy ordinals to original sources — 2026-10-13

Actual predicate rendering now retains original typed condition/transformation source paths internally. Graph candidate construction checks every policy parameter ordinal, value and codec against the private core source inventory, then retains immutable typed metadata beside its original result contract. Original logical subject/key and graph native type remain a distinct unvalued int8 demand. This prevents policy constants or native locator demands from masquerading as query values while retaining existing query-slot refusal and public activation gates.

[Receipt](evidence/security-graph-parameters-20261013/receipt.json) retains the full80-test PostgreSQL unit run, final six graph integration tests, two final source-binding controls, privacy and eleven-package boundary checks. A positive four-policy-ordinal fixture preserves distinct equal string sources, the membership boolean, exact1e2 transformation JSON and separate final subject slot. Seven internal substitution controls refuse wrong source/value/codec/shape/native type bounds. Three retained producer projection shapes remain byte-exact against the previously built producer; all16 producer and8 result controls pass. These are compiler correspondence prerequisites, not native case promotion. Truss's source/epoch/Subject issuers and registered guarded dispatch remain necessary; B-008 and all132 original cases stay open.


### B-008 source epoch comparison independent of each SQL query — 2026-10-13

The existing full graph native fixture archives one exact compiler request/output. It proves lineage for that query but cannot define the universal query engine around reinstalling authority for every SELECT. Core now issues an immutable original source capsule, retaining exact model/policy/ontology/profile/binding/backend bytes while keeping SQL, parameter values and result lineage in the separate existing query context. Source comparison reproduces exact bytes, not a hash credential. The native fixture's current exact request/output comparisons remain enforced.

[Receipt](evidence/security-source-context-20261013/receipt.json) covers273 core tests, all75 admission regressions, five source controls, exact/over serialization bounds, closed output schema and privacy/module checks. Distinct admitted queries and bound parameter values share exact sources while changing query lineage; source whitespace/pin changes refuse correspondence. Existing compiler and manifest validators refuse the capsule as executable request/registration. Registered guarded transport/dispatch must now compose this source comparison with genuine source/Subject/current-authority issuers and per-query typed plan/result lineage. All132 original cases and B-008 remain open.


### B-008 guarded source/query/parameter/result transport — 2026-10-13

The new pure guarded handoff combines the original source capsule with distinct per-query lineage, original checked graph query/guards/result contract, typed policy source ordinals and an unvalued native subject demand. It accepts only the original closed mapping source request; caller actor/epoch/permit flags cannot enter the compiler. Private checked Rust construction and closed source/result transport versions retain ownership. The result inspector regenerates exact producer bytes including the newline and invokes the original complete-batch cell checker. Native admission/release and public activation remain required/blocked.

[Receipt](evidence/security-guarded-transport-20261013/receipt.json) retains four guarded integration tests, six graph regressions, schema/privacy/buffer/module checks and15 actual CLI controls. Mutated SQL/ordinals/source/subject values, omitted newline, changed original request bytes, extra authority fields and later corrupt withheld values refuse. Old/new inspectors reject each other's output; distinct projection queries share source bytes but keep separate query lineage. No native case is promoted. Next compose this exact handoff with Truss's genuine Subject/source/epoch guarded broker and separate installed source comparison; registered semantic coverage, full logical operations, all132 cases and cross-host/native parity remain required.


### B-008 query-use guard increment — 2026-10-14

The PostgreSQL candidate now derives original per-scan guards for all five query-use operators, both join occurrences, disclosed withholding and exact original-action permission. Seven targeted guarded tests and six graph regressions pass; private-constructor compile-fail, source/module inventory and governed artifact checks qualify this pure compiler increment. [Receipt](evidence/security-query-use-guards-20261014/receipt.json). Relational SQL lowering, aggregate result contracts, native pre-filter enforcement and all132 cases remain unfinished; the public activation gate stays closed. The aidev-cus Databricks profile exists, but endpoint inventory currently refuses because no cached credentials are present; the user has been asked to sign in without sharing credentials.


### B-008 aggregate result candidate — 2026-10-14

A separate computed-result candidate now checks original resolved COUNT/SUM domain and argument lineage, exact tokens, COUNT nonnegativity and SUM empty/grouped null semantics. Original result/registration APIs continue to reject its versions. Compiler tests cover changed lineage/facets, incorrect wrappers, negative/null counts and grouped-null sums; direct projection stays on its existing contract. Evidence is retained under `evidence/security-aggregate-results-20261014/`. Relational lowering, native arithmetic/input authorization and the full132-case acceptance suite remain open.

Read-only approved Databricks inventory confirms configured aidev-cus access and an existing private Ashlar development catalog. Historical evidence qualifies ordinary development custody, not scoped end-user security. The sole visible data-gateway warehouse needs a user decision because Ashlar's plan excludes shared/default compute; no SQL, compute configuration or grants were changed during inventory.


### B-008 relational parameter increment — 2026-10-14

The next relational graph path now retains all original query values separately after policy and unvalued Subject slots, with actual occurrence paths and borrowed owner types/values. Tests cover repeated named values containing SQL punctuation, both scanned filter occurrences after a field-equality join, keyset order, field-free COUNT, foreign source/context refusal and retained legacy rejection. Native codecs, SQL lowering/ordinal use, actual guard evaluation, arithmetic and the full132-case acceptance suite remain required. Evidence: `evidence/security-relational-parameters-20261014/`.


### B-008 relational SQL increment — 2026-10-14

The separate immutable SQL candidate now emits the actual relational stages with original parameter/guard/result lineage and explicit operand-validation demands. Grouped disclosure/order consistency and absence-aware grouping prevent arbitrary representative selection or silent absence/null merging. Eighteen compiler integration tests plus six graph regressions qualify declaration correspondence. Seven controlled PostgreSQL18.6 probes qualify selected SQL behavior on a partial synthetic layout; their excluded administrator and missing source/actor/authority admission are recorded, with no native case promotion. Evidence: `evidence/security-relational-sql-20261014/`.

Next: issue and verify original native operand/presence/codec evidence, define a separate guarded relational transport/result inspection, integrate with genuine source/Subject/current-authority dispatch, and expand native parity/refusal coverage across all132 original cases. Registered security activation and the full backend matrix remain open.


### B-008 original operand checking — 2026-10-14

Native SQL candidates now capture uncast original/disclosed operand cells on each primary-permitted scan before relational operations. The core checker validates actual owner-derived occurrences, domains/refinements, absence and exact constant-transform sources. Tests reject invalid later rows, missing/duplicate values, foreign contexts and changed masks while preserving unbounded integer domains. Controlled SQL execution routes actual captures through this checker and proves malformed later inputs refuse before an empty filter can hide them. Evidence: `evidence/security-operand-values-20261014/`.

This closes value-correspondence plumbing, not native capture completeness/authenticity, policy selection, parameter codec qualification or guarded release. Next: separate guarded relational transport/result inspection and genuine native admission/capture integration; all132 original shared/native cases and cross-backend parity remain open.


### B-008 guarded relational handoff — 2026-10-14

The separate0.2 candidate/result transport now preserves original source/query/parameter/operand/result lineage, regenerates exact producer bytes and checks complete supplied cells. Twenty-two integration tests, six graph regressions, private-constructor and module checks, plus12 actual CLI controls qualify original correspondence only. Evidence: `evidence/security-relational-transport-20261014/`.

Next integrate this exact protocol with genuine native source/Subject/current-authority admission, complete operand capture and native codec qualification. Preserve the existing direct dispatcher until the separate relational path is qualified; do not promote supplied hashes/cells into credentials. All132 shared/native cases, backend parity and public security activation remain required.


### B-008 PostgreSQL codec value checks — 2026-10-14

Original parameter and supplied operand values now have separate PostgreSQL18.6 raw-cast correspondence checks. Exact numeric precision/scale/exponent boundaries and NUL refusal preserve original model meaning; valid model integers beyond native storage refuse as unsupported codec values rather than being redefined. Tests distinguish model-domain admission from native value representation. Ten isolated PostgreSQL type probes confirm boundaries; evidence under `evidence/security-pg-codecs-20261014/`.

Native dispatcher integration must check actual target/session settings, run parameter checks before anchors/capture, check genuinely complete operand captures before evaluation and preserve current source/Subject/authority cuts through release. Aggregate arithmetic/output-range qualification and all132 cases remain open.


### B-008 grouped SUM nullability correction — 2026-10-14

The separate aggregate candidate now derives nullable output from actual source availability and selected disclosed null transforms, including nonempty groups whose operands are all null/absent. Required non-null original grouped SUM remains non-null. Security-only optional scalar operator admission keeps original bytes and ordinary subsets intact. Type declaration/emission and cell checking share the owner derivation; copied output objects and narrowed declared nullability refuse. Evidence: `evidence/security-sum-domains-20261014/`. Native optional/null grouping, selected-rule/result arithmetic, current-authority dispatch and all132 required cases remain open.


### B-008 controlled null/absence SQL parity — 2026-10-14

Eleven PostgreSQL18.6 controlled probes now include optional original/disclosed grouped-null SUM and missing-versus-present-null grouping. Actual captures and complete query results pass original owner operand/result checking and exact producer correspondence. Evidence: `evidence/security-null-parity-20261014/`. This is controlled SQL/decoding parity on a partial synthetic layout, not native admission/release or132-case promotion. Genuine0.2 dispatcher integration and the full backend acceptance matrix remain open.


### B-008 explicit native check ordering — 2026-10-14

The original relational handoff now distinguishes initial integrity/Subject/query-use checks from post-operand-validation group checks. Its schema requires closed stage values, exact producer inspection rejects changed stages, and the controlled runner follows them explicitly. Thirty-two targeted compiler/graph tests and eleven controlled native SQL/inspection probes pass. Evidence: `evidence/security-execution-stages-20261014/`. The genuine Truss0.2 host is updating ordering under its existing native cuts; direct0.1 remains intact. Native admission/release, full arithmetic and all132 cases remain open.


### B-008 original132-case coverage audit — 2026-10-14

The original UMF case plan still requires12 shared semantic cases and30 each for pg-raw, Truss, delta-raw and Ashlar. Current allocation labels: shared12 implemented; pg-raw14 implemented,15 not-run and B10 counterexample-found; Truss/Delta/Ashlar30 each not-run. Allocation labels are not fresh native passes. All original IDs, assertions and procedure/evidence fields are retained in `evidence/security-original-case-audit-20261014/`, with exact original case-plan SHA/bytes.

All12 original shared commands were freshly replayed in an exact-source copied snapshot, with assertion IDs, observations, original source hashes and before/after custody checks. Chromium required a local-only server outside the restricted sandbox; S12 used the existing Z3 4.15.4 interpreter. Original owner sources remained unchanged. This qualifies original shared reference semantics and conditional proof, not Weft's compiled/native product acceptance. No case plan/ledger status or native acceptance entry is promoted.

The recorded pg-raw.B10 counterexample concerns ordinary private-fact diagnostic observation and needs a qualified closure, not removal from scope. Recent Truss genuine source/Subject/filter/count/group/stage/publication controls contribute evidence but do not establish every original assertion (for example all collection operations, roles/pools and lifecycle paths). The native owner confirms no Truss promotions. Next allocate/replay exact complete native assertions, preserve distinct source/current-authority/release boundaries, and obtain the dedicated Databricks endpoint/scoped-caller decision for both Delta and Ashlar lanes. All132 remain the completion gate.

### B-008 security candidate descending order — 2026-10-14

The original security SQL parser now retains each ORDER BY direction separately from the historical ASC-only application parser. Descending security queries declare `order.desc`; their owner handoff retains `orderDirections` and the original SQL hash. PostgreSQL relational lowering applies `DESC NULLS FIRST` after materialized eligibility and the original query-use checks; grouped ordering retains its consistency check. Descending keyset/page profiles remain refused, since ascending cursor comparison cannot silently change meaning. Old direct guarded registrations continue to refuse this query.

Validation: 275 core unit tests, 27 guarded transport tests, six graph tests and the original module/spec checks pass. Candidate/result/codec executables were rebuilt for the coordinating Truss chat's exact descending/limit native replay. This compiler evidence is not a native B01/B03 case promotion; the replay and all original 132-case acceptance requirements remain open. Public security activation remains closed.

### B-008 authored traversal profile grammar — 2026-10-14

CONTRACT-007's separate `weft.security.query-profile/0.2-candidate` now has a closed JSON Schema preserving original source/binding pins and requiring one or more explicit `hasRelated` relationship entries. Each entry retains a distinct authored Relationship reference, observation action and target action. Field/Record identity substitution, caller authority flags and other relationship operators are excluded. The historical 0.1 schema and actual profile owner reject this candidate; the new schema establishes structural grammar only, not source/action/native admission or an executable traversal permit. Semantic admission, exact occurrence coverage, correspondence, protected lowering and native fixtures remain required before positive issuance. The original 132-case scope and public security gate remain unchanged.

### B-008 original relationship profile source admission — 2026-10-14

A separate `SecurityQueryProfile::read_relationship_candidate` now admits the 0.2 grammar against exact original policy/ontology/model/binding sources. It verifies declared distinct observation and selected target actions, unique qualified authored relationship identity, selected module ownership, original directed monomorphic independent relationship resolution and source/target membership in the selected ontology/profile. The ordinary 0.1 owner still refuses this candidate. Admission is a retained source artifact, not permission or native compilation: current security query extraction/lowering continues refusing HAS_RELATED.

The tests retain the original authored Truss relationship fixture bytes. Source hash assertions caught the generator's added-newline mismatch; its corrected selected raw files now match both original profile hashes. Tests verify positive original-source admission, unknown relationship/action/domain substitutions, duplicate entries, wrong binding hashes and historical owner rejection. An opaque source-only binding receives an explicitly recomputed test pin; the native fixture's zero placeholder is never admitted as the same bytes. Exact query occurrence coverage, policy truth/obligations, native mapping, SQL/operand/result lineage and protected execution/publication remain unfinished; no original native case is promoted.

### B-008 original public relationship correspondence — 2026-10-14

`GraphBinding::read_relationship_candidate` now separately admits graph-source/correspondence 0.2 against original logical and accepted physical UMF declarations and complete supplied native inventory bytes. It verifies relationship namespace/identity, mapped source/target types, original selected source/target keys and scalar meanings, multiplicities, independent lifecycle, accepted native relationship selectors and exact endpoint rows. Duplicate or substituted relationships/selectors and historical version relabeling refuse; the 0.1 owner continues rejecting the new wrapper. The actual new Truss installation capture supplies the test inputs, including changed native catalog selectors, rather than prior fixture IDs.

Seven graph tests pass, including original-version regressions and the new exact captured correspondence controls. Module/spec checks pass. The first test exposed the Truss physical request's absent `selectedModuleIds`; the Weft fixture explicitly selects original physical module `m` while preserving accepted document bytes/pins. Hosts must supply that explicit selection when assembling the wrapper. These are supplied source/correspondence checks, not authenticated live assessment or permission. Public relationship SQL, source/observation/terminal policy obligations, capture/codec/inspector and guarded release remain to be connected before an executable positive candidate can be issued. No original case promotion or public activation.

### B-008 SQL-owned relationship occurrences and action obligations — 2026-10-14

Separate relationship query resolution now retains actual WHERE HAS_RELATED predicate positions from the original owner IR. Each terminal has an independent compiler occurrence and original typed authored key query uses. The selected 0.2 profile must cover exactly the used authored relationships; missing or unused entries refuse. Per-scan obligation derivation retains the source's ordinary action plus the distinct observation action, and the terminal's independently selected read action, including original complete private policy dependencies. Extra actions share the existing cumulative work/text ledger. No caller-supplied scan, action or key annotation is accepted.

`with_security_relationship_context` separately composes these original source/profile/query obligations. The historical mapping-context path and ordinary security query resolver continue refusing HAS_RELATED. The positive authored-source test verifies source read, genuine observation-rule dependency and terminal read; unused relationship profiles and old resolver substitutions refuse. These contexts are immutable compiler source lineage, not native permission. Graph/native SQL dispatch, protected witness rendering, parameter/operand capture and codec/result/protected publication remain unfinished; no executable traversal or case promotion is claimed.

### B-008 one-hop protected relationship SQL and separate transports — 2026-10-14

The separate relationship producer now emits HAS_RELATED as EXISTS over the actual mapped typed public edge, the source's independently read-eligible population, its observation-action truth, and an independently read-eligible terminal occurrence with the original typed key comparison. Source read and observation truth remain distinct; a declared observation action with no permit yields no public witness. Indeterminate observation refuses before query evaluation. Public edge integrity checks cover all selected relation edges, including hidden endpoint/type/key corruption. Complete terminal-key operands are captured and validated before comparison. EXISTS preserves source bag cardinality rather than joining each edge into output rows.

Explicit entrypoints `prepare_guarded_relationship_candidate`, `inspect_guarded_relationship_result` and binaries `security_relationship_candidate`, `security_relationship_result`, `security_relationship_codecs` use the separate relationship context. The output remains the closed relational candidate/result 0.2 envelope with four required native duties and original source capsule 0.1. Historical producer/inspector/codec paths continue refusing the new profile/wrapper. Original producer-byte reconstruction, policy/Subject/query parameter separation, codec checks and result-cell correspondence are retained. Empty supplied operand/result batches in component tests are declarations, not evidence of a complete native population or authenticated selection.

Both original permitted and observation-denied fresh Truss requests emit candidates. Seven graph and 28 guarded transport tests pass, including exact closed outputs, old-protocol rejection and SQL/newline tamper refusal; module/spec checks pass. The first native profile selects directed independent one-hop string-key relationships with unrestricted 0..* participation on both sides. Finite/minimum degrees refuse until complete distinct-neighbor checks are implemented; inverse, association paths and RELATED_KEYS remain separately unfinished. Protected native caller/source/epoch/fact/publication composition and all original 132 acceptance cases remain mandatory. No original case promotion or public activation is claimed.


### Mutation API reconciliation and current Truss/Python bridge — 2026-10-14

The frozen mutation-owner source package was reconciled onto current main,
preserving its security and Truss runtime implementations. Existing main's
CONTRACT-006/008 and US-009 identities remain unchanged. Imported mutation
artifacts receive CONTRACT-009, US-010/011, TD-010/011 and STP-010/011 with
explicit FR-23–26 allocation; older evidence remains historical source scope.
No whole dirty checkout was adopted.

The original products model was accepted/exported through actual Truss APIs.
The current adapter reuses original runtime65/tagged mapping/scalar semantics,
not older PostgreSQL17 SQL. Native Python now exposes original action compiler
construction, explicit current renderer registration and original bounded
selection lowering. All input source bytes/revisions/binding hashes remain
checked. The first native attempt caught a wrong physical `version` assumption;
current layout uses `ver`, verified in the native owner and corrected before the
passing replay.

Eight mutation compiler, ten embedding and eight historical lowering tests pass.
Native Python passes32 independent golden/oracle cases. A fresh same-store
PostgreSQL18.6 replay compiles the original UPDATE, runs complete preflight and
typed selection through the Python bridge, and applies its assignment through
the public version-checked Truss SDK: bar becomes Foo and baz remains keep.
Overflow, stale revision, substituted binding and malformed filtered-out Key
controls refuse. The ordinary person retains no raw table SELECT. Native raw
selection uses an excluded test assessor; protected RR/current authority/bulk
sealing remains independently owned work, with no B-008 completion claim.
Evidence: [current native receipt](evidence/mutation-current-truss-20261014/receipt.json)
and [source/transport manifest](evidence/mutation-current-truss-20261014/manifest.json).

The final installable abi3 wheel was built with the pinned toolchain, installed offline into an isolated target, and passed the32-case Python oracle plus the same-store native replay on Python3.12.14/macOS arm64. The bridge also rejects caller SQL/principal/plugin members, unknown handles and PostgreSQL NUL parameters. No package was published.


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


### Original AST query property candidate — 2026-10-15

The separate `security_mapping_transport::prepare_security_query_properties`
function and `weft-core` example `security_query_properties` consume the existing
closed original mapping request and emit `weft.security.query-properties/0.1-candidate`.
The closed schema is `security-query-properties-v0.1-candidate.schema.json`.
No existing QueryUse, mapping handoff, candidate SQL or registered compiler wire
is extended or reinterpreted. The report has securityReady:false and authority:false.

Each actual resolved expression position retains its path, scan occurrence,
qualified target/Field, original revision-qualified Field identity and logical
type. Equal predicates and both sides of join equality require equality;
lexicographic comparisons and ordering require order; grouping requires equality.
Equal-valued or repeated Field expressions remain distinct occurrences. Hidden
Fields used only by filters/order/join/group remain dependencies. Source scans
survive field-free COUNT(*); contradictory predicates do not erase dependencies.
Directed one-hop relationship terminal keys derive equality at their own tN scan
when the owner invokes derive with an admitted relationship context.

SUM retains actual argument/output numeric type and the resolved mode/action,
but required:null/propertyStatus:unsupported. No numeric transform property is
invented or classified as equality. The actual legacy mode and originalAction
are marked modeSource:legacy-ontology-query-profile. Full exact source capsule,
SQL, source/query hashes and pins retain their provenance. This report does not
resolve the current UMF catalog, canonical stored Field or storageTransform:
the original catalog/owner consumer must resolve those independently. A selected
compatible transform property never supplies a read/native permit.

Derivation refuses missing original requirements and stale identities; caller
property lists, paths, modes or actions are not accepted inputs. Work retains at
most4096 dependency occurrences with a conservative text charge and16MB final
serialized report ceiling. The ceiling is an endpoint/output bound, not a
preallocation, CPU or whole-compiler resource theorem. Existing4MB request and
source admission limits remain. JSON facets remain opaque original type content.

Actual parser refusal of COUNT(DISTINCT), LIMIT0 and WHERE FALSE remains explicit
in this first candidate. They require language/IR and null/duplicate/zero-result
conformance, with no optimization that bypasses permissions. Numeric SUM rules,
current catalog/storage-transform source consumption, effective disclosure and
original action/current-authority/native before-dispatch checks remain required.
This component does not close P03/B08, promote an original native case, change
any case assertion/allocation, or activate protected execution.


Current-source dependency: this first report requires original legacy0.1 ontology/
query-profile admission. Current UMF catalog/ontology0.2 source bytes must never be
rewritten into a synthetic legacy policy to obtain this context. A later explicit
current-source admission or policy-independent application-AST lineage surface
must derive requirements from the actual current query/model, retain unknown
source bytes and omit unestablished modes/actions. This report does not establish
current catalog compatibility by carrying a legacy-source mode label.


### Policy-independent application lineage candidate — 2026-10-15

The distinct `application_lineage::prepare` API and `application_lineage` core
example accept `weft-application-lineage/0.1-candidate` and emit the closed
`weft.application.lineage/0.1-candidate` artifact. Request/report schemas live in
`application-lineage-request-v0.1-candidate.schema.json` and
`application-lineage-v0.1-candidate.schema.json`. A private immutable result is
constructed only by resolving the original pinned core0.8 modules and SQL through
the existing application parser/resolver. No SecurityLogicalPlan, policy0.1 adapter
or synthetic legacy policy is constructed. Old registrations/transports stay
separate. No executable plugin, SQL override or caller AST/required/mode/action
input is accepted.

A shared internal traversal of the resolved application AST derives original
occurrence paths, scan/Field identities with revisions, equality/order properties
and actual numeric SUM argument/output types. It is used by both the earlier
legacy property report and this new source-only report. Legacy output bytes must
remain unchanged. Self-join sides and repeated expressions are distinct, hidden
Fields retain uses, terminal keys retain their own tN equality occurrence, and
COUNT(*) preserves source scans with no invented Field dependency. Unsupported
SUM properties remain required:null/propertyStatus:unsupported. ASC/DESC direction
and required capabilities are preserved consistently; unsupported descending
cursor combinations refuse. COUNT(DISTINCT), LIMIT0 and WHERE FALSE still require
language/IR/conformance work and remain refused.

The report retains exact sourceRequestJson/hash, original model pins, SQL hash,
actual applicationPlanJson/hash, scan identities and uses. It contains no mode,
modeSource or originalAction. Optional external opaque source records retain
role, declared document/revision identity, media type, exact UTF8 bytes and digest
inside the original request. Their status is always uninterpreted; matching a
supplied digest proves local byte correspondence, not ownership, current policy
meaning, storageTransform, effective disposition, original action, key/runtime
scope, authentication or native permission. Unknown opaque source content is never
parsed as a compiler override. Original model content remains retained; selected
unestablished core element meaning still refuses.

Limits are4MB input bytes,16 opaque sources,4096 dependency occurrences,8MB
parameter replication/retained-use/serialized-plan budgets and16MB serialized
report output. The bounded writer refuses without partial output. These are
explicit endpoint/component bounds, not a whole-process allocation/CPU theorem.
The pure parameter preflight is reused from the original mapping transport.

The UMF owner's explicitly new named-current-stored-token-r1 fixture is retained
unchanged with source SHA2216a7ad3093c90437db035fb337d21ac67589dee5eb148b0eb4d7fd1ff75059.
Its current policy0.2, ontology0.2, catalog0.1 and storage owner values are extracted
with recorded compact UTF8 JSON encoding; the exact whole fixture bytes are also
retained as an opaque source. Five actual SQL scenarios independently derive
operation/property lineage, including selfjoin/group, output-omitted token order,
unsupported numeric SUM and field-free COUNT. Expected stored uses remain authored
test expectations, never producer inputs or attestations. Opaque expectation
mutations cannot change AST uses. The original unnamed UMF fixture is preserved
as a refusal: Weft never adds names/members to make it queryable.

UMF must separately consume these original issued requirements against its actual
catalog/storage-owner program and effective read policy. No current catalog
compatibility, native before-dispatch/source/current authority, protected release,
P03/B08 completion or original backend case promotion follows from this artifact.
All original assertions, allocations and deadlines remain mandatory.


### Selected availability regression repair — 2026-10-15

An independent owner replay exposed that the source-only0.1 parser could resolve
an absent-allowed scalar as an ideal nonnull Field while its dependency report
omitted actual availability. Predicate, projection, hidden order, selfjoin, group
and SUM argument paths could therefore emit misleading domain lineage. This is a
source-report counterexample, not evidence of a native data leak or permission.
The original six-case and expanded eight-case before witnesses are preserved under
`tests/lineage-availability/`; the old frozen producer passed only the required
predicate and field-free count controls. The earlier legacy property report also
accepted an original absent salary source; its actual before output is retained.

Both `application_lineage::prepare` and `security_query_properties::derive` now
require actual selected Field descriptors to have availability required. The
shared check resolves each original scan Record/Field membership from Catalog,
then inspects the original descriptor; it does not infer presence from the ideal
LogicalType.nullable flag or the existence of a predicate. It covers plain output
Fields as well as every hidden/operator/aggregate/terminal-key dependency before
any report is returned. Unrepresented availability refuses with
WFT-APPLICATION-LINEAGE-AVAILABILITY and no partial stdout. Absence is not NULL;
nullable/absent source semantics need a separately explicit qualified descriptor
and protocol before these reports can claim them.

Field-free COUNT(*) remains supported as structural lineage: it selects no Field,
retains every original scan and source duty, and does not blanket reject an unused
optional Record member. A required projection similarly survives an unselected
optional sibling. Neither positive control establishes native source completeness,
policy eligibility, current authority or guarded release.

All existing all-required0.1 reports, current five-source scenarios and historical
mapping outputs remain byte-identical. There is no envelope/schema/registration
upgrade or compatibility fallback. Fresh292 core tests,16 boundary controls and
10 host availability controls pass; exact original before witnesses, repaired
binary/source pins and current/legacy correspondence evidence are retained under
`evidence/lineage-availability-20261015/`. Frozen old producers remain historical
counterexample evidence, not qualification of selected absence. Full P03/B08 and
all native backend acceptance duties remain open. The in-progress0.2 grammar is
kept separate from this repair.

## Native Truss0.66 query compatibility — 2026-10-15

Backend truss.postgresql.runtime0.4.36 separately admits the exact native0.66
manifest99ae7218 and UMF0.8/support0.17 binding. Native0.65/backend0.4.35
remains unchanged and refuses the new cohort; the new cohort refuses old bindings.
The original named-query-core-r1 logical revision is preserved. Standalone scalar
declarations have no invented Record membership or property/storage identity; a
query selecting declaration-only clear refuses. The additive manifest capability
schema.declarativeScalarFields describes catalog compatibility only. Historical
mutation/security profiles retain their original runtime pins.

The local weft-sql0.4.30 arm64 wheel passed five actual public-SDK native queries
(projection, COUNT, predicate, group and selfjoin), declaration-only refusal and
ordinary caller raw-object privilege denial. The frozen Truss50307788 SDK used
an explicit candidate BACKEND0.4.36 metadata override; this is not qualification
of an unmodified committed SDK. Exact receipts and local wheel hash are under
evidence/truss066-20261015/. The native setup applies the original five owner
Records through public APIs and preserves original source/readback hashes.
This is ordinary catalog/query integration, not installed protection/Subject/
current-query authority. B-008 remains open. Distribution publication is separate.

Follow-up paired replay passed all seven checks with the exact committed Truss
SDK query-runtime module99f9a129 over frozen native50307788. There is no SDK
metadata override in this replay; both source pins are explicit in
evidence/truss066-20261015/committed-sdk-query-receipt.json. Build-only GitHub
Actions run38754435498 targets compiler77639ba for five wheel platforms; no
tag/release publication was requested. Full85 PostgreSQL unit run is still in
progress (84 passed, exhaustive decimal-domain case pending).

Final manifest admission hardening: backend0.4.36 requires the exact committed
99ae7218 native manifest digest in addition to its runtime version and binding
digest. A freshly rehashed binding with a substituted manifest digest refuses
without SQL; both new native-cohort tests pass after this tightening. Earlier
77639ba wheel/replay receipts remain historical before-hardening evidence.
A rebuilt wheel and matching native replay are pending; no released asset is
qualified by the earlier build-only run. Older registrations are unchanged.

Final50464e0 source passed the seven native checks with committed SDK99f9a129
using a freshly installed development-profile wheel (SHA6878b8ca). Full85
PostgreSQL unit regression tests passed on the pre-hash-guard source; the final
manifest-guard source separately passed both focused tests. Exact logs and
final-build.json retain this composition rather than claiming a fresh full run
after the guard. The release-profile wheel96b0fa8d fails macOS import with
mis-aligned LINKEDIT string pool and is explicitly unqualified. The final-source
five-platform build-only run38754989288 remains pending; no package release
is published. Truss received source, replacement wheel pins and this limitation.
Ordinary query integration passed; current-policy/native protection and B-008
completion remain open.

## B-008 source-owned read requirements and COUNT candidate — 2026-10-15

The separate application-lineage0.2 candidate now extends the original application
parser/resolver with COUNT DISTINCT, flat WHERE Boolean conjuncts and LIMIT0,
retaining original row/Field/relationship dependencies through false/zero output.
CONTRACT-006 defines the closed request/report/IR and readRequirements contract;
TD-009 and STP-009 retain the implementation and verification scope. No alternate
policy compiler, caller AST/action/mode authority or backend fallback is admitted.
Old0.1/property/mapping bytes and availability refusals are unchanged; existing
registrations refuse the new IR before backend binding/lowering callbacks.

Fresh current-main integration passes300 core tests, workspace all-target check,
16 boundary controls, both retained native066 cohort tests, six candidate paired
reports and all current/legacy/availability host controls. Original owner schemas
pass68 artifacts/77 schemas/52 allocated criteria/636 fixture scenarios (document
checks only). The actual UMF consumer independently passes262 controls over18
fresh original compiler reports; all18 report bytes match its reviewed candidate.
Evidence and exact source/binary pins are under evidence/lineage02-20261015/.

The bounded count model passes six UNSAT/four SAT sensitivity checks. Eleven
previously retained PostgreSQL18.6 VALUES primitives remain byte-corresponding
source evidence, not a fresh compiled protected-query execution. Required source
Fields stay required; absent/null ambiguity, unsupported SUM properties and
unsupported shapes retain refusals/explicit unsupported status. Native original
scalar access and historical mutation pins stay unchanged.

Next connect these exact issued read requirements to the owner's current read/
capability program and original authenticated native admission. Actual caller/
Subject/current source and eligible-population custody, compiler-to-native/result
lineage, guarded release and every original132-case shared/native gate remain
mandatory. This increment promotes no acceptance case and does not complete B-008.

## B-008 optional Python/WASM read-requirement bridge — 2026-10-15

The application-lineage-candidate feature now exposes the same original core
issuer through pure runtime composition, native Python and browser WASM.
CONTRACT-006 defines the exact success/refusal surface; TD-009/STP-009 retain
module ownership, parity and no-feature checks. No alternate parser, native
executor or policy mode/action interpretation is introduced. Existing compiler
entrypoints and released native066 source50464e0 remain separately pinned.

A freshly installed dev-profile Python wheel and actual Chromium151.0.7922.34 WASM
each pass11 exact0.1/0.2 report fixtures and5 malformed/duplicate/override/oversize
refusals. Both ordinary compile entrypoints still refuse lineage envelopes.
Chromium test initialization uses exact local WASM/JS; host I/O is fenced after
initialization, with no page errors/external requests. A separate default-feature
wheel omits the optional API and retains the ordinary runtime API. One runtime
bridge test, explicit feature checks and16 boundary controls pass. Exact source,
wheel/WASM/JS hashes and receipts are under evidence/lineage02-bridge-20261015/.

This closes embedding of issued source read requirements only. Original132 native
security cases, genuine caller/source/current authority and guarded end-to-end
result release remain open. No artifact is published and no acceptance case is
promoted. Next compose the owner's current whole-query read program and actual
native admission against these original issued bytes rather than caller assertions.

### Native066 workflow completion

GitHub Actions38754989288 completed successfully at exact50464e0: all five
platform wheel build/install/import/API smoke jobs passed and the publication job
was skipped. evidence/truss066-20261015/workflow-final.json records the observed
source/job outcomes. This supersedes the earlier pending build status, not the
local96b0 import failure. Truss separately reports adopter, numeric and upgrade/
query checks against the actual workflow Mac artifact; those are owner-reported
until its exact retained receipts are independently consumed here. No current
protected-security or newer opt-in lineage release claim follows. Tag/publication
is coordinated separately and was not performed by this increment.

## B-008 original typed preparation lifetime — 2026-10-15

The core preparation now retains actual Catalog, ApplicationPlan, original sort
directions and exact original request UTF8. Immutable accessors expose those
original values without reconstructing a plan from report JSON. The source-owned
Python preparation holds the same core result, with no report/dict constructor,
mutation, dynamic state, subclass, copy or pickle path. Its native receivers reject
forged dictionaries/strings/duck types. JSON/report protocols remain exact.

The direction vector is essential: the older Plan order list alone omits ASC/DESC,
and span-aligned opposite-direction queries can have identical Plan bytes. A new
native lowerer must consume the full issued preparation and per-occurrence owner
value descriptors, preserving output disclosure separately from hidden original
operands. Existing registrations still reject0.2 IR before callbacks, including
older SQL syntax transported under the new protocol. Native profile/issuer/action
approval is not inferred from object possession, hashes or source declarations.
Private native source preparations must stay inside the trusted host boundary;
authority:false does not permit publication of their retained host inputs.

Fresh301 core tests, runtime/doc tests,16 boundary controls,31 installed preparation
controls,16 Python and16 real-Chromium checks pass. An explicitly dual-feature
wheel coexists with ordinary native066 public queries and passes8 native controls:
reports cannot become compile requests, new syntax remains refused and ordinary
raw-object access is denied. This is the old ordinary route, not0.2 typed-plan
lowering or current protection. The actual source CLI passes445 independent UMF
consumer checks over27 reports; selected source pins are retained, not a hermetic
or native-proof claim. All old/candidate/property/mapping/availability report
bytes remain unchanged. Exact receipts are under evidence/issued-context-20261015/.

Next implement a separate current-source designation and SQL lowering cohort over
this context, with joint ontology/catalog modes, per-use effective domains and
exact original-action bindings. A new original owner source is required for the
original-authorized positives; r1/r2 remain unchanged. Actual native caller/source/
current-authority and guarded result-release gates, every original132 backend case
and expanded owner coverage remain open. No acceptance case is promoted.

Concurrent integration retains the reviewed native66 mutation fix e389d60 and
release preparation f87939f. A fresh dual-feature0.4.31 wheel passes31 object and16
JSON controls after composing those exact peer sources; workspace all-target
checking is retained separately. The8 ordinary-native controls remain qualified
to the earlier0.4.30 image and its frozen SDK, rather than being relabeled as a
fresh0.4.31 SDK qualification. No current/native protection claim follows.

### Current-source original-action designation — 2026-10-15

B-008 remains open. The separate `weft.current-query.designation-profile/0.1-candidate`
now binds the original typed preparation, source/model/binding bytes and exact
per-occurrence original-authorized actions. Read, update, reverse and undeclared
actions cannot substitute. Missing designation remains unresolved rather than a
permission default. The source fixture is UMF92f228e's unchanged current query-value
source. Weft authors the designation independently of owner authoring-intent fields.
The profile/report are separate from every existing backend registration and the
legacy relationship profile; HasRelated designation currently refuses.

Evidence: `evidence/current-query-designation-20261015/receipt.json`. All305 core
tests,17 original UMF-purpose correspondence controls,12 CLI correspondence/refusal
controls,18 installed Python-wheel designation controls and12 actual Chromium/WASM
controls pass. The existing31 immutable-object and16 Python JSON controls pass.
Workspace all-targets, module boundaries and governed-spec checks pass. The local
wheel is an unpublished development build with the unchanged0.4.31 version; it does
not replace the published release. No native/security execution claim follows.

Next integrate UMF's original-source joint mode/effective-value route semantics
with this issued designation and retained original typed context, then implement
the separately registered native lowering/consumer path. Read disclosure, hidden
original use and primary row eligibility must remain distinct. Actual authenticated
caller/source/Subject/current-authority admission, paired runtime scopes and guarded
result release remain required. All original132 shared/native backend cases remain
in scope; the expanded UMF gate stays45 accepted/113 unimplemented/158 required.

### Native environment and issued-object custody — 2026-10-15

B-008 remains open under every original132-case obligation. The user-approved
Databricks test catalog now has actual one-caller UC/Delta row-filter and column-
mask evidence:17 statements pass and the disposable namespace is removed. SQL
version2026.39 is observed after the run. This resolves the earlier untested
managed-table/filter/mask platform prerequisite for that endpoint, not source/
caller isolation, compiler/native integration, retention or Ashlar qualification.
The second configured caller still fails authentication. See
`evidence/current-security-primitives-20261015/receipt.json` for sanitized source
pins, actual checks and their precise limits.

The same increment verifies11 Rust public API doctests. Current query designation
JSON cannot be imported as an issued object; its original preparation cannot be
replaced or outlived. No executable semantics or published wheel version changes.

Truss confirms native66 has no current-policy0.2 held issuer/consumer API. The
Truss owner has queued a separate host-issued opaque query context plus private
PostgreSQL admission/release guard, using only qualified portions of existing
caller/Subject/source/role/table/DDL cut helpers. It must span complete per-resource
original-action preflight, application SQL, decoding and guarded release. This is
an assigned implementation boundary, not an available API or acceptance evidence.
Weft retains ownership of original use/action designation, mode/domain consumption,
SQL and result lineage. UMF's route inspector and Weft's draft correspondence test
remain outside this committed increment until the upstream interface is pinned.

### Original operand pairing for current-query lowering — 2026-10-15

B-008 remains open. The new immutable core operand ledger is issued only from the
actual current-query designation and original typed preparation. It preserves
ordered comparison pairs, self-join scan identities, literal/named-parameter
payloads/types, per-use action bindings, grouping/distinct occurrences and complete
ASC/DESC vectors. It retains Boolean predicates and COUNT duties under FALSE and
LIMIT0; unsupported SUM refuses without returning a partial ledger. The complete
Field occurrence set must match the original shared AST visitor.

Eight new source-backed controls pass. Composed with main535b5c7, the full core
suite passes313 tests and public API custody passes12 doctests. Workspace,
module-boundary and governed-spec evidence is retained in
`evidence/current-query-operands-20261015/receipt.json`. The separate local mutation
cohort is preserved. No executable protocol/report, published wheel or registered
backend changes, and no original132-case promotion follows.

Next use this actual typed operand context to check UMF's conditional effective
value routes per occurrence, preserve projection disclosure independently, and
lower through the separately admitted native consumer. Logical family equality
alone is insufficient for protected comparison/group/distinct across rows. Exact
runtime key/transform scope, real current source/caller/Subject cut, complete
resource-dependent action preflight and guarded result release remain mandatory.

### Pinned current-source route interoperability — 2026-10-15

B-008 remains open under all original132 shared/native obligations. The actual
Weft issued lineage/designation images now interoperate with UMF's committed
c57d283 source-route inspector under exact source/producer pins. The full1683-file
src/spec/package/lock inventory is checked before/after; source overlays and earlier
unqualified producers refuse. Fifty-one correspondence/constraint controls and4
additional provenance refusals pass. Conditions, use identities, domains and
read-versus-hidden-original meanings are retained. Unsupported SUM remains
unsupported. See `evidence/current-route-interoperability-20261015/receipt.json`.

This is source/conditional-route interoperability only. It does not select runtime
branches, authorize an original action or produce native protected SQL. Next add
Python `prepare_current_query()` on the actual immutable original preparation,
retaining that same object and shared Rust context while deriving designation and
typed operand ledger together. Native consumers must use the issued object's
actual context, not import report/plan JSON as authority. UMF's next runtime route
activation and Truss's held caller/Subject/source/current-cut issuer remain separate
unmet implementation gates. The concurrent local mutation/binding cohort is
preserved and cannot substitute for this native security path.

### Same-original Python current query handoff — 2026-10-15

B-008 remains open. Actual Python parent.prepare_current_query() now derives the
source-bound designation and typed operand ledger together, returns a frozen
non-importable child and retains the actual original parent plus shared immutable
Rust context. The Rust with_context consumer hook supplies that actual original
context and temporary typed declarations, rather than importing report/plan JSON.
No SQL/AST reparse or global Field retyping occurs. Original object identity is
checked independently of identical source bytes. Native consumers must use actual
native fields, not overridable Python methods or diagnostic JSON.

Five runtime tests,3 runtime API doctests and46 installed-wheel handoff controls
pass, alongside18 designation/31 parent-object/16 JSON regressions. Workspace and
WASM builds pass;16 real Chromium bridge controls pass. Evidence/source/wheel pins
are retained in `evidence/current-query-object-20261015/receipt.json`. This is an
unpublished0.4.32 development wheel, not a replacement for the published release.
The existing reports and registered backend behavior remain unchanged.

Next implement the separately registered current-source PostgreSQL scalar
consumer through this typed hook and Truss's genuine held native issuer. Initial
required singular String/signed64 application scalar qualification is a first
subset only; all original132 shared/native cases and full raw/graph goal remain
required. Unsupported uses must refuse the whole lowering without empty-selection
bypass. UMF owns actual read-rule/route activation; exact separate original-action
permission, per-use/cross-row runtime scopes, original native source/caller/Subject
cut, SQL/result lineage and guarded release remain unmet. The new source object
is not evidence that any of those gates passed.


### Conditional typed PostgreSQL scalar compiler — 2026-10-15

B-008 remains open under all original132 native/shared obligations and the
expanded45 accepted/113 unimplemented/158 required gate. The opt-in
current-postgresql-scalar-candidate now lowers actual same-original typed
operands to PostgreSQL SQL over private declared scalar columns, including
separate masked read DTO and raw predicate columns. The closed binding and result
metadata are defined in CONTRACT-006. Arbitrary materialization declarations are
conditional premises; SQL emission does not admit source policy or native access.

Seven targeted Rust controls and nine actual PostgreSQL18.6 SQL/result cases pass,
including bags, grouping/order/count, bound parameters and FALSE/LIMIT0. The
Python parent/current-child custody regression passes46 controls. Native fixture
execution uses a privileged local connection and synthetic eligibility/mode/scope
declarations, including counterfactual source modes. It is SQL refinement only;
no original protected backend case is promoted. The disposable server is stopped.
Evidence is recorded under evidence/current-scalar-sql-20261015/.

The unpublished0.4.32 development wheel enables the separate candidate feature;
it does not replace the released0.4.32 artifact. Truss's actual database-derived
capture/currentSubject issuer ABI and guarded execution/release remain pending.
The UMF computation driver stays on hold until that native boundary is concrete.
The user authorizes isolated Databricks tests under client_dev and cleanup;
the approved second caller connection is still unresolved.

An independent ordinary-role PG18.6 oracle also passes17 positive and4 pre-execution refusal controls over UTF8 C-collation and signed64 edges. Removing eligibility filtering changes the expected bag and is detected. Its owned container is removed. These checks share the conditional premise qualification above and confer no native policy admission.

The full core/PostgreSQL/runtime test invocation passes, including the exhaustive existing decimal storage-home matrix and Rust API doctests. Workspace all-target checking, module boundaries and governed-spec checks also pass; see the exact logs and test-result ledger in the receipt.


### Same-original computation input and private row lineage — 2026-10-15

B-008 remains open under all132 original backend obligations and the expanded
45/113/158 gate. A no-argument native Python method now derives exact operation,
operand and UMF route requests inside the retained Rust typed context. The new
separate0.2 current-row compiler carries native opaque TEXT keys through direct
projection/inner-join SQL, with separate private-column and public-cell source
metadata. It never imports a caller ledger or establishes native source access.

The actual Python -> Rust -> PostgreSQL18.6 fixture passes13 row/bridge controls,
including duplicate masked rows, both join contributors, exact staged cell
correspondence, FALSE/LIMIT0, no imported packet/foreign self and three aggregate
refusals. The server stops and its lease is released. Ten targeted scalar Rust
controls, the original typed packet core control,46 Python custody controls and
nine previous scalar native cases pass. All nine predecessor compiled artifacts,
SQL hashes and result bags are identical with the new wheel. The native fixtures
use privileged local connections and synthetic stage/mode/scope declarations;
these prove conditional SQL/lineage only. No original native case is promoted.

Evidence is under evidence/current-row-lineage-20261015/. The unchanged-version
0.4.32 development wheel is unpublished and has separate pins. Truss agreed the
closed0.2 TEXT-key carrier and Python bridge interface; its actual original-source
installation, authenticated capture/Subject/current-cut issuer and guarded final
release remain incomplete. Grouped contributor bags, COUNT/DISTINCT and remaining
scalar/aggregate domains are next required compiler work, currently refused by
this0.2 path. They are not removed from B-008 scope.


### Complete grouped contributor bags and counts — 2026-10-15

B-008 remains open under all132 original shared/native obligations and expanded
45 accepted/113 unimplemented/158 required coverage. The separate0.3 current
compiler now emits complete private bags of joined source-key tuples for grouped
queries, COUNT and COUNT DISTINCT, including singleton nonaggregate and explicit
empty-global-aggregate bags. Separate cell/group metadata retains original paths
and Field identities; native hosts must hold exact statement/result custody and
validate complete contribution and disclosure before release. Properties remain
ordinary scalar columns; JSONB is used only for the private lineage carrier.

Thirteen targeted Rust controls and20 actual same-original Python -> Rust ->
PostgreSQL18.6 controls pass (17 positive queries,2 version refusals,1 deliberately
incorrect contribution oracle). The incorrect sideband keeps a public distinct
count equal while changing the expected joined bag; this proves the finite oracle
can detect it, not that an unimplemented native guard rejected it. The privileged
fixtures use synthetic mode/scope/value declarations, including counterfactual
salary group/distinct modes. No original protected backend case is promoted.
All completed disposable servers stop and release their leases.

The previous scalar9 and row-lineage13 check records/SQL hashes/result bags are
identical under the new unpublished0.4.32 development wheel;46 Python custody
controls pass. Evidence is under evidence/current-aggregate-lineage-20261015/.
Truss agreed the closed contributor bag/cell/group ABI and explicit native
consumer registration boundary. Actual source/Subject/current-cut admission,
complete native facts, transform/runtime proof and guarded final release remain
Truss/native work; Ashlar/Databricks qualification and the second caller are open.

Next retain native current-query integration and implement remaining SUM/exact
scalar semantics. UMF owner agrees a separate positive live-original signed64 SUM
source revision with an independent original-authorized action, and necessary
nullity/presence dependency only. Nullity is not additive transform preservation.
A separately versioned capability must preserve old SUM-prohibited source and
unsupported-factory negatives; nullable mathematical output requires exact native
arbitrary-precision accumulation. Disclosed/transformed SUM and remaining domains
stay required and unsupported until their actual semantics are established.

UMF main9d0e8758 retains five actual prior0.2 Weft packet -> original Bun child supplied-cut cases and26 focused verifier controls. The retained README/root verification were inspected; that external evidence qualifies the prior wheel/source pins only and grants no native SQL/caller/current-authority or backend acceptance.


### Explicit original integer SUM context — 2026-10-15

B-008 remains open under all132 original shared/native cases and expanded
45 accepted/113 unimplemented/158 required coverage. Designation-profile/report0.2
now selects live-original-integer-sum and exact per-operation action bindings;
a SUM action is not automatically assigned to COUNT DISTINCT. The typed factory
preserves original signed64 input and contextual mathematical Integer output:
nullable globally, required in nonempty groups. Native-input0.2 derives necessary
nullity requirements only under that capability. This does not claim additive
transform preservation or alter old unsupported property reports.

Separate SQLconsumer0.4 requires the new context, original-authorized untransformed
signed64 input/domain and exact action. Qualified bigint SUM accumulates as exact
numeric and uses explicit nullable/required text codecs, retaining complete
contributor bags. All older consumers reject the new context even for COUNT or
empty work, and the new consumer rejects old contexts. Disclosed/transformed SUM,
other numeric widths/domains and wider required operations remain unsupported
and in scope. No native permission or registration is issued by the compiler.

The original source is independently pinned UMF1bb8ec4768 SHA
d9a87aa6b230eacf38b7b56451725cdf226a492610c2872c585008badd8c7b15.
Weft authors its own profile without owner intent/expectation inputs, preserves
exact role JSON slices and checks selected Core numeric tokens. Old source and
factory negatives are unchanged. The actual conditional PG18.6 corpus passes20
controls, including sums beyond both int64 limits, global null versus grouped
required output, exact joined bags and version/mode/action refusal. The server
stops; module/shim/wheel/runner pins match before and after cleanup. The three
predecessor native ledgers (9/13/20) remain identical;46 Python custody checks pass.
All317 core tests,15 targeted scalar controls, actual Chromium13 metadata controls,
workspace, module16 and governed-schema checks pass. Evidence is retained under
evidence/current-original-sum-20261015/. These are declared-stage compiler proofs,
not authenticated policy/current-authority or protected backend acceptance.

Truss reports its first finite ordinary-caller masked/raw-predicate query workflow
on main9a495c23, still requiring exact new standard-wheel qualification and wider
original cases. Published Weft0.4.32 lacks the optional current-query feature;
prepare0.4.33 standard platform wheels with explicit current-query and ordinary
PG18 features plus exact-wheel API/ABI2 smoke gates. Do not overwrite32 or infer
native security support from compiler method availability.


### Standard wheel current-query feature selection — 2026-10-15

Prepare0.4.33 with both ordinary PG18 and current-postgresql-scalar-candidate
features. Published0.4.32 omitted the actual CQP class; no existing release/tag
is overwritten. Every standard platform must install its exact wheel and smoke
actual original-context custody plus conditional ABI2/3/4 and no-downgrade; retain
source/wheel/module/shim pins. Pin all build checkouts to the event SHA, not a
moving branch. Publication must require five matching source/version/wheel proof
records. Build-only workflow dispatch does not publish a release.

This repairs the package dependency for Truss's first guarded scalar query.
It does not qualify native security. Independent UMF00e5c912 replay confirms the
same ordinary caller can still receive raw protected r1/r2 through granted native
query_objects/query_binding. Truss owns the all-path activation fix; original
pg-raw.B10/AC5 and all132 backend obligations stay open. Final paired release and
exact-wheel/native qualification remain pending this preparation.

Local optimized0.4.33 standard-feature wheel passes actual ABI2/3/4/no-downgrade
smoke,46 Python custody controls and original native65/66/67 mutation smoke.
The first local optimized artifact failed Mach-O load validation; retaining final
extension line tables with strip:none fixes the observed pinned-toolchain case
without changing release optimization. The failed artifact is not qualified.
Module/spec checks pass. Exact five-platform/source-pinned build-only CI and
paired native33 workflow qualification remain pending; nothing is published.


### Databricks current consumer preparation — 2026-10-15

B-008 remains open under all132 original shared/native cases. CONTRACT-006 now
records the separate Databricks current-value ABI and original CQP handoff,
managed provider obligations, exact String/signed64/finite integer codecs and
complete contributor bags. STP-009 records actual compiler/native gates. This is
a proposed implementation contract, not a registered backend or security claim.

Six actual constant-only SQL probes on the approved aidev-cus managed warehouse
pass, including exact18446744073709551614 SUM, empty SUM/bag/count, duplicate
contributors, binary comparison, named exact parameter and qualified builtins.
The independent finite receipt oracle rejects six semantic corruptions. No user
schema/table was created, no source data was queried, and no second caller was
simulated. Evidence is in evidence/current-databricks-semantics-20261015/.

Next implement the Rust typed Databricks consumer, Python scoped bridge, exact
version/refusal controls and independent emitted-SQL native corpus. Ashlar owns
the actual held authority/provider adapter; UC policy/time-travel compatibility
and second approved caller remain required native gaps. Complete native source,
caller/current authority and guarded result release cannot be replaced by these
six dialect observations or a finite representation declaration.


### Exact wheel promotion after tag-ref publication failure — 2026-10-15

All five wheel jobs in tag run38807410042 pass against v0.4.33/cc943a2. The
publish job fails before release creation because checkout by immutable commit
with default shallow depth does not retain refs/tags/v0.4.33. Fetch full tag refs
in the publish checkout; keep the exact source/tag correspondence guard.

The optional promote_run_id workflow input reuses existing wheel artifacts only
after exact same-repository/tag/source/workflow provenance and all five successful
platform jobs are checked. A failed publish job may be recovered without rebuilding
or overwriting a public release. Native module/shim/wheel hashes and identical
current-query checks are validated again from the downloaded wheel bytes. The
verifier is fetched from the immutable workflow commit; compiler/release source
remains the selected tag. Two tests include14 corrupt-provenance/job controls.
This changes release delivery only and does not promote B-008 security acceptance.


### Published standard0.4.33 exact-wheel verification — 2026-10-15

Release https://github.com/DocumentDrivenDX/weft/releases/tag/v0.4.33 is public.
Promotion38808676764 passes using the original five tag-run38807410042 artifacts;
no recompilation or retagging occurs during promotion. All five downloaded public
wheels equal their verified tag-run bytes. Sourcecc943a2 and workflowe98bd19,
embedded native module/shim digests, public smoke reports and SHA256SUMS match.
The actually installed public ARMwheel passes all four current-query API checks.

The ARMnative module is identical to the independently replayed CI module, while
the public wheel archive digest differs. Truss must explicitly adopt public pins;
adding ambiguous same-module cohorts or automatic ABI3/4 registration is invalid.
Evidence is in evidence/release033-published-20261015/. Publishing repairs the
package dependency only: all132 original B-008 cases and full native security
qualification remain open. Databricks typed compiler implementation is next.


### Typed Databricks current scalar compiler and Python bridge — 2026-10-15

B-008 remains open under all132 original shared/native cases. The separate pure
Databricks consumer now emits current-value SQL through the actual original CQP
context, with exact Delta relation/version declarations, bound values/scan labels,
explicit binary collation, complete private contributor bags and guarded exact
original signed64 SUM. Native Python exposes the two explicit profile APIs under
current-databricks-scalar-candidate. Existing registered backends do not activate
these methods or migrate bindings. The package version0.4.34 identifies an
unpublished development wheel; public0.4.33 remains unchanged.

Nine new Rust tests and the complete existing Databricks package suite pass.
The native Python bridge passes32 new controls and46 existing custody controls.
The actual same-original Python/Rust -> managed Databricks corpus passes25 controls
(22 positive emitted-query cases, wrong-contributor oracle,2profile refusals),
including exact sums beyond signed64, empty/global nullable versus grouped required,
join bags, masks and parameter injection-as-data. Its dedicated client_dev schema
is removed. The native tests use synthetic materialization/mode/scope declarations
and original logical fixtures; no accepted Ashlar installation or actual native
policy/source/caller authority is asserted. All original132 case IDs remain intact.

Evidence is in evidence/current-databricks-sql-20261015/. Next integrate the actual
Ashlar held provider and original UMF decisions, then complete genuine native
source/caller/current authority, two-caller and guarded-release acceptance. The
compiler's declared premises and JSON digest cannot substitute for those gates.
Broader nullable/structured/numeric/operator/disclosed-SUM scope remains required.


Standard Python wheel selection now explicitly includes the Databricks candidate
feature. Release builds read their pinned source pyproject features, and every
new standard platform runs32 exact-wheel Databricks bridge/version/refusal controls.
Publication requires five matching Databricks reports when that feature is selected,
with identical wheel/module/shim/source/version pins to the existing PostgreSQL
smoke receipts. Historical0.4.33 artifacts need no Databricks report. A local
no-extra-feature standard build has all five wheel member payloads identical to
the explicitly enabled, managed-tested development wheel. This is archive/member
correspondence, not another native database run or security qualification.
