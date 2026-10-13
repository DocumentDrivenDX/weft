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
