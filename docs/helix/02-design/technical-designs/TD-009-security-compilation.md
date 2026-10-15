---
ddx:
  id: TD-009
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: FEAT-006
      kind: informed_by
    - id: SD-004
      kind: informed_by
    - id: CONTRACT-006
      kind: informed_by
    - id: CONTRACT-007
      kind: informed_by
---

# TD-009: Security foundation assurance

**Story:** [[US-009-security-compilation]]. **Parent solution:** [[SD-004-conformance-evolution]].

## Technical Approach and Assurance Level

CONTRACT-007 and authored UMF policy/ontology meaning govern the foundation.
US-009-AC1–AC4 remain open for full product acceptance. B-009 R5 provides precise
reviewed specifications plus bounded executable SMT checks of selected foundation
properties; it does not claim arbitrary compiler refinement or native enforcement.
Use `z3-solver==4.15.4.0`, pinned in host-only reliability requirements. Z3 checks
negated safety formulas for UNSAT and explicit success/failure witnesses for SAT;
unknown, timeout, missing solver or mismatched version fails the assurance gate.
[Official Z3 guide](https://microsoft.github.io/z3guide/programming/Z3%20Python/Introduction/)
governs solver result interpretation. Tool choice is a design pin, not evidence
that the tool or formulas have already run.

## State, Authority and Transitions

State comprises pinned source identity, admitted rule truths, selected field
protection/dispositions and public activation state. Truths are True/False/Unknown;
caller facts are simulations. Admission → evaluation → composition is a pure
foundation flow. Public 0.5 compilation transitions directly to unsupported
refusal before backend construction and cannot emit a security artifact.
Native authentication, complete cuts, current authority, publication and final
release are host assumptions for future integration, never simulator outputs.

## Properties and Bounded Executable Scope

| Stable property | Authority and safety assertion | Bounds / correspondence |
| --- | --- | --- |
| WFT-FM-001 | CONTRACT-007: public security requests cannot produce executable artifacts | Actual compile.rs refusal tests; guard/factory negative controls; no model proof of native behavior |
| WFT-FM-002 | CONTRACT-007: unknown scope-matching truth makes composition indeterminate with empty disclosure | One permit, one require, one forbid; three truth values each; all 27 combinations compared to authored oracle and actual Rust compose |
| WFT-FM-003 | CONTRACT-007: permit needs true permit, true require and false forbid; denial has no disclosure | Same finite rule set; denied/allowed SAT witnesses and deliberately broken permit gate |
| WFT-FM-004 | CONTRACT-007: withheld dominates; incompatible selected transform identities conflict | Selected one field, up to three active permit dispositions; abstract exact normalized identities; Rust exact-literal/custody tests supply correspondence |
| WFT-FM-005 | CONTRACT-007: a stale source/catalog cannot be admitted as the same prepared plan | Actual source/pin/admission tests; precise custody invariant rather than SMT hash-collision claim |

FM004 starts after the rule truth gate has admitted at least one active permit.
Slot0 means that an active permit contributes no disposition for the selected
field, not an absent grant. The Rust correspondence fixture uses three true
permit rules and one true requirement for all432 disposition/protection tuples;
it compares the independent authored table and reversed rule order. Three
transform equivalence classes cover up to three distinct exact identities.

Formal abstraction separately encodes authored decision/disclosure rules and a
bounded counterpart of implementation behavior. R5 records formulas, source
fingerprints, tool/version, SAT witnesses, UNSAT results and broken-model SAT
counterexamples. Real Rust tests check oracle correspondence and exact literals;
model success alone cannot establish implementation refinement. Transform identity
abstraction excludes scalar arithmetic, hash collision resistance and arbitrary
AST induction. Population/expression/resource bounds remain actual Rust test and
review obligations, not hidden solver assumptions.

## Liveness and Exclusions

Finite SMT queries use a fixed timeout and treat unknown as failure. Pure Rust
simulation has existing finite row/expression/payload budgets, reviewed and tested
at boundaries; no temporal native-service liveness is proved. Authentication,
Record-backed native evidence, arbitrary source/IR/evaluator equivalence, physical
lowering, private predicate effects and protected release remain deferred.

## Component Changes and Interfaces

Retain the existing blocked B-008 source, independent truth/composition oracles
and source fixtures. R5 adds host-only formulas/checker/evidence, not a new public
security activation path. CONTRACT-007 owns exact APIs. Core remains deterministic
and does not import solver, configuration, logger, storage or credentials.

## Testing

STP-009 distinguishes component admission/simulation tests from full acceptance.
US-008-AC6 (owned by STP-008) covers exhaustive allocation and scoped formal evidence. Record missing
correspondence or negative-control failures as failures. Every property must have
an authority, code/test mapping and explicit limits in retained R5 evidence.

## Migration, Trade-offs and Rollback

No native schema migration. Bounded assurance gives inspectable counterexamples
at low execution cost but cannot prove larger populations, native custody or
arbitrary policy semantics. Withdraw a property claim when its mapping changes;
keep public activation blocked until separately governed native acceptance passes.

## R5 executable gate

`python scripts/reliability/gate.py` consumes CONTRACT-008 configuration. It runs
the pinned solver and exact selected Rust correspondence tests; missing tools,
unknown/timeout, mismatched solver versions, zero/ignored/failed selected tests
or an altered oracle fail qualification. `formal.py` contains separate authored
tables and implementation abstractions, negated UNSAT checks, SAT witnesses and
deliberately broken permit/withheld counterexamples. Host-only dependencies
remain outside the core. Retained evidence maps each stable property to its
actual code, test and fixture fingerprints; no full FR19–22 acceptance is claimed.

## Remaining protected-read realization — 2026-10-10

The B-008 continuation in the implementation plan sequences remaining
US-009-AC1–AC4 work. Existing original admission/query/handoff and separately
typed candidate graph source/IR/dependency/composition/mixed-witness simulation
are reusable foundation components. They remain conditional on supplied facts;
no draft plan authenticates a subject or releases a protected result.

S00 must reconcile exact upstream security contracts in the UMF security
worktree, choose source/request/response/matching versions and update
CONTRACT-001/002/003/005 and schemas before dependent implementation. The
blocked-only0.3 response cannot silently become executable. Graph query bridging,
closed semantic capability matching and compiled disclosure/result domains are
open contract decisions; this design does not invent their normative payloads.

Component ownership remains the reviewed architecture/module map. Core owns
private-constructor requirements derived from the actual admitted plan/profile,
complete scan-action-rule inventory and ordered resolved outputs. Backends own
physical mappings, interpreted coverage and SQL/result carriers. Hosts own
original actors, complete private-fact issuers/current cuts, installed inventory,
publication and final release. Boundary changes precede feature code and run
`bun run modules:check`; operational configuration follows CONTRACT-008 outside
core. Existing B-009 scoped concern overrides require review on activation.

The compiler must retain all selected requirements, including unsupported
COUNT/SUM/relationship cases, and refuse incomplete coverage. Conservative
possible-disposition envelopes never replace runtime evaluation of every scoped
rule. Stored facts and trusted context remain distinct; exact transform/output
identity and original/null/absent/withheld distinctions survive result admission.
Candidate graph consumers require actual query/scan integration and whole-cut
admission, not merely the existing single-rule simulator.

Extend formal properties only with reviewed authority/bounds and independent
Rust/SQL correspondence. New graph coverage, owner-derived result coverage,
capability completeness and release ordering are separate obligations from the
R5 finite truth/disposition model. Counterexamples include omitted rule/action/
output occurrence, false-grant mapping, incomplete empty cuts, stale source and
release before guard completion. Native concurrency evidence must use real
barriers/observations and cannot be substituted by an abstract liveness claim.

STP-009 allocates new positive/negative tests; B-008 S05–S12 qualify each physical
profile separately. Rollback withdraws the changed capability/activation tuple,
retains prior protection and refuses stale profiles. Public activation remains
blocked until its exact source/result/native contract and all required evidence
pass. Authentication, write execution and backend installation stay externally
owned prerequisites rather than new compiler side effects.

Capability admission requires coherent complete scan/action and whole-application
bundles, compatibility across selected capabilities and refusal before backend
invocation when coverage is fragmented or selected meaning is unknown. Native
authority/query-carrier populations require bidirectional selected-Key and
normalized-value correspondence before filtering; omitted/extra/duplicate rows,
same-typed Key-component swaps and stale values are explicit refusal controls.
Current main version/artifact-ID collisions are an integration prerequisite;
checkpoint foundation claims do not qualify the reconciled composition.


## Current-main security registration correspondence

CONTRACT-006 now governs main’s dedicated security 0.1/0.2 protocols; CONTRACT-007 retains the blocked compile 0.5 compatibility envelope. The finite no-executable-output property includes the new registration protocol: security 0.2 may call its explicit trusted registration factory once after source/query/profile admission, but SecurityBackend::lower remains uncalled and the response refuses unsupported lowering. Compatibility 0.5 and security 0.1 never invoke either backend factory. Generic security 0.2 has no implicit registration and refuses BACKEND-REQUIRED. The prior callback prohibition is historical and does not erase main’s admitted registration component. New protocol-matrix and main registration/owner-refusal tests provide actual Rust correspondence; native authentication, enforcement and release remain unqualified.


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

## Application read-intent and count candidate implementation — 2026-10-15

CONTRACT-006 governs the separate0.2 request/report/IR. application_syntax gates
the extensions behind parse_lineage_candidate; application_resolve constructs the
original plan, retaining source/join/terminal scans before Boolean or LIMIT effects.
application_property_uses remains the common operation dependency traversal.
New private application_read_requirements derives projections, row scans and
hidden/operator occurrences against actual Catalog membership/descriptors. No
policy adapter, synthesized SecurityLogicalPlan or user-authored action is added.
Older parser entrypoints retain refusals; original backend dispatch rejects the
new IR before its mapping callback. Existing PostgreSQL/Databricks lowerers have
explicit unsupported arms rather than accidentally acquiring new SQL semantics.

This realizes component prerequisites for US-009-AC1/2/3, not their full native
acceptance. Module inventories must include the new private source, shared enum
match sites and evidence hosts; bun run modules:check is the boundary gate.
Rollback disables the0.2 request version while retaining0.1 and exact historical
reports. Current-policy/native admission, actual caller/Subject/current-cut
custody, complete eligible population and guarded result release remain required.

The COUNT model checks0..4 paired rows over mathematical equality classes with
complete population, bidirectional equality and nullity agreement. Six UNSAT
checks establish only those bounded conditional implications; four SAT sensitivity
checks expose missing equality/nullity/population/native-nonnull premises. A
required-argument specialization still needs an actual native nonnull guard.
The model is not a Rust/SQL refinement proof or a pre-storage plaintext equality
claim. Separate PostgreSQL18.6 VALUES tests cover count/NULL/duplicate/equality
primitives and intentional counterexamples; they install no protected policy and
admit no nullable/absent UMF Field. Preserve these scope distinctions in all
compiler/native evidence and do not combine component counts into acceptance.

### Optional native/browser lineage embedding

CONTRACT-006's application-lineage-candidate feature adds a thin pure runtime
composition and Python/WASM bindings to the same core issuer. It supplies exact
report bytes rather than rebuilding a query AST in Python or JavaScript. New
interfaces are gated in all three crates; existing release feature defaults and
compile entrypoints stay separate. Errors preserve the original Diagnostic code
and become host-language exceptions, with no partial report. Module ownership
adds only explicit runtime/python/wasm API inventories; core has no host I/O.

### Native producer result lifetime

The runtime wraps the original core preparation in a private-field immutable
value. Python's feature-gated frozen class owns that runtime value; the binding
has no direct core dependency and accepts source requests only through the
original producer. It has no constructor/serde/pickle/copy path from a report.
The native json receiver requires the actual issued type. This preserves a real
in-process producer result while the host performs separate original source and
native authority checks; it is not a native credential or issuer approval.
CONTRACT-006 governs this custody surface. Browser and JSON report protocols
remain unchanged. Hosts must preserve trusted procedure references rather than
consult caller-overridable Python attributes as authentication.

The private core preparation now retains the already resolved Catalog and Plan,
plus typed original order directions. This keeps original typed context available
for a future explicit current-source lowerer without a second parser or a JSON
reconstruction. DESC/ASC cannot be inferred from the older Plan.order Field list.
The future lowerer must accept the complete preparation and separately bind an
owner-derived effective query-value descriptor; it must preserve original core
semantics and refuse operand/domain/key/runtime incompatibility. Retaining model/
plan allocations extends their lifetime to the preparation's lifetime; hosts
must account for the number/lifetime of held preparations. Report/input bounds
remain endpoint/component bounds rather than a process-wide heap theorem.

## Source-bound current-query designation implementation

CONTRACT-006 defines the independent current profile/artifact. New core module
current_query_designation borrows the original Catalog/Plan/directions/request
from ApplicationLineage; it never resolves a second SQL AST. Exact profile/source
linkage, actual Record/Field membership, complete purpose declarations and unique
Field/operator mappings precede issue. The shared operation visitor produces
per-occurrence action requirements. The runtime/Python object and WASM function
reuse that same issuer under application-lineage-candidate; default release
features stay separate. Update core/runtime/python/wasm inventories and run
bun run modules:check before integration.

This introduces only a bounded Rust interpretation of action-purpose declarations
for Weft's owned operation roles. Verify it against original UMF purpose checking
and preserve narrower identifier limits. It does not implement mode composition,
policy evaluation, storage effects, effective values or native authority. The
UMF callable needed next is a private source-derived per-use route inspection over
its original program/owner plus issued Field/operator requirements; it should
report applicable route/domain/scope conditions without minting a Weft designation
or caller-selected permission. Native registration then supplies actual effective
source/domain and paired-key/runtime evidence for the separately versioned lowerer.

Formal-methods scope: this slice binds symbols/occurrences and validates bounded
sets, with original-owner differential and mutation controls. It adds no transform
or relational algebra theorem; existing conditional source/value and native proof
obligations remain open. Do not substitute this structural evidence for SQL/native
refinement. Rollback disables the new profile/issuer, preserving original0.1/0.2
lineage bytes and old registrations' refusals.

#### Typed original operand ledger — 2026-10-15

The core current-query operand factory borrows the actual issued designation and
its original Plan. Its private owner exposes read-only operations/operands, with
no deserialization or replacement API. Equality retains two ordered operands;
lexicographic inputs retain alternating column/value pairs in original component
order. Group/order/distinct uses stay distinct by occurrence path. Order directions
come from the complete original preparation, not the older Plan representation or
its digest. Fields, parameter names/payloads and logical types remain borrowed
from the original resolved context; the factory does not synthesize descriptors.

The factory checks its complete Field-occurrence set against the shared original
visitor and caps operations/Field occurrences at4096. Original request and
resolved-parameter budgets still apply; literal payloads are not cloned. It
refuses unsupported aggregates/expressions before returning any ledger. This is
finite additional bookkeeping, not a total heap/CPU bound for the full pipeline.

This is the next typed input for effective-domain/native lowering. The operation
vector is an occurrence ledger, not execution order; original Plan stages and
separate read-disclosure requirements remain governing. UMF must supply original
source-derived conditional routes. Native source/caller/Subject/current-cut
admission and guarded release remain separate mandatory gates. No backend is
registered or activated by this increment.
