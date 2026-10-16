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

## Application lineage0.2 candidate verification

The v02_tests and v02_read_requirements_tests modules in application_lineage.rs
provide component coverage for US-009-AC1/2/3, under CONTRACT-006 and TD-009.
They assert original occurrence-sensitive row/Field/relationship read intents,
projection/hidden/RHS-join/aggregate identity and availability, retained FALSE/
LIMIT0 dependencies, invalid hidden-name/parameter refusal, absent argument
refusal, SUM unsupported status, new IR identity and old protocol/parser refusal.
The backend lineage_candidate_refusal_tests control asserts refusal before mapping.
These tests cite @covers US-009-AC1/2/3; all four full story criteria remain open.

Six paired fixtures under tests/lineage-v02/fixtures cover projection, hidden
reads, distinct selfjoin, field-free count, required COUNT DISTINCT and unsupported
SUM. Rebuild every report from exact original request bytes and compare the whole
output. Also replay every original0.1 and legacy property request byte-for-byte;
no version relabel, adapter replacement or expected-output rewrite is permitted.
A fresh core unit run, workspace all-target check and module-boundary controls
are integration gates. Independent owner consumer evidence must identify the
actual compiler binary and exact report hashes, not fixture expectations as input.

prove-count-correspondence.py requires z3-solver4.15.4.0 and asserts six UNSAT/four
SAT results. native-count-distinct.py requires actual PostgreSQL18.6 and verifies
11 VALUES primitives/counterexamples, exact int8 counts and stopped-server cleanup.
Neither qualifies native compiled protected queries, nullable-source admission or
US-009-AC4. Every original132 shared/native backend case remains allocated with
its original source/caller/current-authority/guarded-publication obligations.

### Installed lineage bridge parity

Opt-in embedding tests under tests/lineage-v02/python-bridge.py and
browser-bridge.mjs cite US-009-AC1/2/3 and require eleven original0.1/0.2 reports
to match exact saved bytes, repeated browser calls to remain deterministic and
five malformed/duplicate/override/over-budget inputs to throw without a report.
Ordinary compile_json must refuse the lineage envelope. Browser initialization
must load actual WASM in Chromium; after initialization host fetch/XHR/WebSocket/
Worker access is fenced, page errors and external requests fail the test. Record
exact wheel/WASM/JS hashes, build features and browser version. The no-feature
wheel must omit the bridge. These are binding/source-lineage component checks,
not native policy/caller/current-cut/guarded-release acceptance.

### Issued Python preparation controls

The installed issued-python.py test cites US-009-AC1/2/3 and compares all11
retained report fixtures through the actual typed preparation and JSON API.
Direct report/dict/no-argument constructors, object.__new__, attribute/dynamic
state changes, copy/deepcopy/pickle, and dictionary/string/duck-type native
receivers must refuse. The real object must remain unchanged. Whitespace and
Unicode/NUL/duplicate-content opaque source bytes remain exact and uninterpreted;
legitimate producer invocation on caller source still reports authority:false.
Malformed and over-budget source inputs refuse. A Rust compile-fail example
checks private-field construction. Default builds omit the typed surface. These
controls do not establish native policy or caller/current-generation authority.

Typed-context verification compares the retained actual Plan serialization to
applicationPlanJson, actual Catalog inputs/pins to original request/report bytes,
join/zero-limit occurrences and mixed DESC/ASC order alignment. The old-backend
callback trap now consumes the actual issued preparation rather than resolving a
second plan, and includes an older-syntax query inside the new0.2 protocol.

The native-issued.py harness explicitly builds both application-lineage-candidate
and truss-postgresql-pg18, exercising ordinary native066 query compatibility with
the original accepted source/binding. Lineage reports must fail as native compile
requests; COUNT DISTINCT/FALSE/LIMIT0 remain refused by the original SQL profile,
while source read duties survive. This installs no current protection policy and
must not promote a native security case or treat the preparation as permission.

The ordinary native coexistence harness deliberately keeps the existing native
compiler route separate; it checks original source pins/SQL and refusal boundaries,
not lowering the new typed0.2 Plan. A future current-native lowerer must consume
that exact issued context and per-use effective descriptors directly. Native source
preparations may contain private host inputs; releasing those reports is outside
the guarded result contract and must remain prohibited in the actual adapter.

## Current-query designation verification

New core current_query_designation tests and tests/current-query-designation
cite US-009-AC1/2/3. Seven original committed UMF query-value source requests must
issue exact action paths: salary predicate and token order designate their own
actions; other operators retain null designation. Original integer salary model/
plan descriptors remain unchanged despite its separate masked String DTO.

Reject unrelated read/update/reverse/vault/unknown actions, missing/duplicate/
unknown/incompatible purpose declarations, mixed legacy profile versions, wrong
source/model/binding pins, missing/duplicate selected source roles, duplicate
bindings, nonmember standalone DTO references and caller mode/action/AST additions.
False/zero output retains hidden requirements; empty bindings never grant a mode.
The original UMF purpose checker independently compares known-subset role outcomes,
including an action whose spelling is unrelated to its declared purpose. Record
original owner/binary/source pins and distinguish purpose compatibility from whole
policy admission. Native Python and actual Chromium compare exact issued bytes and
refusal behavior, while all prior lineage/property/mapping/availability outputs
stay exact. Native P03/B08, current authority and all132 backend cases remain open.

### Real Databricks platform prerequisites — 2026-10-15

`tests/lineage-v02/databricks-security-primitives.py` uses one genuinely
SQL-authenticated configured caller against an already running Unity Catalog SQL
warehouse. It creates a unique schema in the user-approved test catalog, a managed
Delta table with two synthetic resources, a current-user row filter and a constant
BIGINT column mask. It checks filtered projection/count/SUM/distinct/self-join/
predicate and masked projection/predicate/group/SUM results. A `finally` cleanup
drops only the fresh owned schema and verifies its absence. The fresh-namespace
check precedes creation; failed creation still enters cleanup. It refuses truncated
results, unexpected rows, failed statements and an endpoint that is not already
running. The runner never starts compute or changes caller grants.

Seventeen statements pass, including fresh-namespace and cleanup controls. SQL
version2026.39 is observed afterward on the same configured endpoint. The receipt
contains no caller identity, endpoint ID or catalog/schema name; namespace and SQL
text are represented by digests. Original private identity output remains outside
the repository. This is one-caller platform evidence, not compiled Weft security,
Ashlar layout, second-caller isolation or physical-retention qualification. No
original132-case status changes. The second configured connection still fails
authentication; an approved scoped caller connection remains required.

Eleven Rust public API doctests also pass, including three designation custody
controls: report deserialization, replacement of original context and detachment
from its lifetime cannot compile. These are Rust API boundaries inside the trusted
host, not protection against unsafe host code or proof of native authority.
Evidence: `../../04-build/evidence/current-security-primitives-20261015/receipt.json`.

### Current-query operand correspondence — 2026-10-15

Eight actual-source Rust controls cover self-join pairing and DISTINCT input,
hidden Integer salary versus separate read DTO, original literal/parameter
payloads and types, FALSE/LIMIT0 preflight with field-free COUNT, atomic unsupported
SUM refusal, opposite direction vectors with byte-identical serialized plans, and
separate group/order/distinct occurrences of one Field, and ordered tuple column/parameter pairing. Public API compile-fail
coverage refuses importing a serialized operand ledger. Tests consume actual
ApplicationLineage and CurrentQueryDesignations factories rather than authored
replacement ASTs. The original shared visitor must match every Field occurrence.

The composed main base includes the separate local mutation cohort unchanged.
All313 core tests and12 public API doctests pass, with workspace all-target,
module-boundary and governed-spec verification recorded separately. These checks
prove original operand custody only; they execute no current protected SQL and
promote none of the original132 shared/native cases. Per-use effective domains,
paired/cross-row runtime scopes and genuine native admission/release remain open.

### Pinned UMF route interoperability — 2026-10-15

`tests/current-query-designation/value-route-conformance.ts` executes the exact
committed Weft lineage/designation producer images against UMF's reviewed source-
route inspector at commitc57d283ece30326bcb06607ca5d2384fc69ddef4. The test pins both
producer hashes and the complete1683-file src/spec/package/lock source inventory,
rejects extra source files and checks all source bytes before and after execution.
Use an exported original source tree matching `umf-source-manifest.json`, with
locked dependencies available separately. The observed embedding is Bun1.4.2 on
macOSarm64; the pinned CLI images are platform-specific evidence, not a universal
binary-build claim.

Fifty-one controls pass:10 original occurrences (including explicit unsupported
SUM),36 closed-shape mode/action/authority override refusals, and5 joint-constraint/
missing-designation/direction-context controls. Each inspected route retains its
exact compiler path, scan, target, Field identity, operator, operation and required
properties. Original policy read/membership conditions and full core domain
identities/declarations must match. The salary read DTO remains String while its
hidden predicate retains original Integer meaning; token queries retain the live
stored opaque Field and nonselectable pre-storage input. Conflicting/prohibited
modes and missing per-use actions cannot become permission defaults.

`provenance-refusals.py` adds4 actual pre-import refusals: unqualified producer,
modified source bytes, extra source file and changed source manifest. No partial
report is written. Temporary counterfactual source copies are removed. Mode-source
mutations are diagnostic counterfactuals, not accepted policy revisions. Neither
inspection nor these tests activate a branch, evaluate current permissions, emit
protected SQL or qualify native admission. The original132-case completion gate remains
open; individual statuses do not change. Evidence:
`../../04-build/evidence/current-route-interoperability-20261015/receipt.json`.

### Same-original Python current query handoff — 2026-10-15

Five runtime controls pass, including actual pointer identity across original/
designation/operand objects, retained context after parent drop, rejection before
callback of corrupted declaration/count or substituted source with byte-identical
Plan, and atomic unsupported/legacy refusal. Three runtime API doctests reject
private construction, report import and escaping scoped operand borrows.

The actual installed macOSarm64 Python abi3 development wheel passes46 handoff
controls in `tests/current-query-designation/issued-current-python.py`: exact
original parent identity, six original designation goldens, actual operation
counts, identical-byte distinct parent separation, retention after GC, frozen
constructor/attribute/subclass/copy/pickle refusals, actual native receiver checks,
no override inputs, legacy compiler/report refusal, and native descriptor behavior
under class-method replacement. Existing18 designation,31 original-object and16
JSON bridge controls pass. Unsupported SUM refuses; a prohibited source can still
be represented as non-authoritative preparation and is not admitted for SQL.

Workspace all-target checking passes. The same runtime ownership change builds
for WASM and passes16 original JSON-lineage controls in actual Chromium151.0.7922.34.
The child object is Python/Rust only; no WASM object or new native consumer is
claimed. Module-boundary/spec checks and source/wheel/module correspondence are
recorded in `../../04-build/evidence/current-query-object-20261015/receipt.json`.
The development wheel retains version0.4.32 and does not replace that published
release. No original132 acceptance case is promoted.


### Current scalar SQL refinement — 2026-10-15

Exercise actual issued context -> typed lowering -> Python native extension ->
PostgreSQL18.6, with declared private fixture values. Cover independent masked
projection/raw predicate, self-join bags, COUNT DISTINCT, grouping/order, COUNT,
FALSE/LIMIT0, named parameters and injection-shaped parameter data. Assert actual
text OIDs and exact result bags, mathematical COUNT metadata, and cleanup. Rust
controls additionally require exact pins, closed objects, complete occurrences,
paired scopes, original actions, type mismatch and unsupported/prohibited refusal.
These conditional stage premises do not qualify policy correctness, authenticated
caller/current authority, Truss issuer, paired transform runtime or protected
publication. Keep all original132 backend obligations and expanded158 gate open.


### Native input packet and row-contribution candidate — 2026-10-15

Derive operation/route packets from an actual retained Python child with no
arguments. Check original source/designation/action/type/literal/direction and
UMF required-capability correspondence; imported packet and foreign-self calls
refuse. Execute actual0.2 SQL on PG18.6: duplicate masked projection rows must
retain distinct private source keys, joins retain both actual staged contributors,
and public cell/source associations must match private fixture rows. FALSE and
LIMIT0 retain preflight requirements. Verify TEXT OIDs/closed carrier/scan order,
0.1 backward behavior and cross-version refusal. Explicit aggregate refusal
remains pending grouped-bag work. Stop the disposable server on every exit.
The finite fixture proves conditional SQL and source contribution only; no
source-policy correctness, authenticated caller/current authority or guarded
public result release is qualified by it.


### Grouped and count contribution bags — 2026-10-15

Exercise actual same-original Python child -> typed0.3 SQL -> PostgreSQL18.6
against exact fixture bags: COUNT, COUNT DISTINCT (String/Integer), self-join
multiplicity, grouped projection/count, grouped joined distinct, hidden grouping,
multiple group fields, empty global/grouped input, FALSE/LIMIT0, raw predicates,
named parameters and singleton nonaggregate bags. Verify each cell from declared
staged values, every grouping source and mathematical COUNT metadata. A deliberate
wrong right-hand source key must change the expected joined bag even while the
public distinct count stays equal. This is a counterfactual oracle control, not
an implemented native-guard rejection. Require cross-version refusal and both
predecessor native corpora unchanged, exact text OIDs and disposable-server cleanup.
Synthetic declarations/counterfactual salary modes do not qualify UMF/native
source permission, authenticated caller/current authority or final publication.


### Original integer SUM and explicit capability boundaries — 2026-10-15

Verify the separate committed original SUM source, exact role-byte/model-token
custody and independently authored operation-scoped Weft profile. Core tests must
reject unknown/missing/legacy capabilities, absent SUM action and unsupported
signedness/width/domains, without borrowing SUM action for COUNT DISTINCT.
Actual Python -> Rust -> PostgreSQL18.6 checks must cover sums beyond positive and
negative int64, exact joined multiplicity, global empty scalar null, no empty group,
nonnullable grouped sums, separate identical-public-value bags, SUM/count/distinct,
FALSE/LIMIT0 and typed parameters. Fixed SQL signatures independently determine
expected source routes, operations, grouping and result types; producer omissions
must not self-validate. No public salary mask column is staged for original-only
SUM. Verify exact text OIDs, complete private bags and native module/shim/wheel/
runner pins before and after cleanup. The three predecessor corpora and legacy
factory negatives remain unchanged; browser designation parity is metadata only.
These supplied stages are conditional fixtures, not native policy/current authority
or guarded result-release acceptance. Disclosed/transformed SUM and remaining
specified domains stay open.


## Databricks same-original current consumer — required implementation gate

CONTRACT-006's separately versioned Databricks handoff is proposed, not activated.
Retain the original132-case catalog and all US-009-AC1–AC4 rows. Required tests:

- Actual Rust/Python same-original CQP custody, no imported ledger/ordinary fallback,
  profile1 versus explicit SUM profile2, PostgreSQL/old/extra binding rejection,
  exact source/model/static binding pins and duplicate/missing per-use refusal.
- Managed Databricks compiler-emitted SQL against independent expected bags:
  direct/join/group/order/COUNT/DISTINCT/original SUM, exact signed 64-bit extremes,
  counts versus complete contributor multiplicity, global null/grouped required,
  empty/FALSE/LIMIT0, exact parameters and Unicode binary semantics.
- Deliberate contributor deletion/duplication/scan swap, correct public count with
  wrong private bag, wrong effective domain/scope/action and masked/raw confusion.
  Actual held native executor must reject corruption before public release.
- Exact native STRING metadata/codecs, private-column confinement, grouped source
  correspondence, nonempty numeric overflow versus empty SUM, null/absent stage
  refusal and no partial rows on decode/cancellation/deadline/overflow failures.
- Generated aliases and escaped native identifiers, case-insensitive native name
  collisions, hostile backticks/quotes, parameter names and builtin resolution.
- Genuine Ashlar original publication/table UUID/version/schema vector, managed UC
  policy/time-travel compatibility, actual two-caller isolation, opening/closing
  drift/revocation, retained snapshots and alternative native access paths.

The six retained constant-only managed probes and six receipt-corruption controls
qualify their finite dialect/receipt oracle only. They do not implement these
compiler/native tests or close any protected backend acceptance case. Local Spark
is not a substitute for Databricks identity, Unity Catalog or source authority.


### Conditional Databricks component evidence — 2026-10-15

Nine new Rust current-consumer tests and the existing Databricks package tests
pass. The real Python bridge passes32 new descriptor/import/override controls
and46 predecessor custody controls. Actual compiler-emitted managed SQL passes
22 finite direct/group/join/count/SUM cases plus a wrong-contributor negative
oracle and two profile refusals. The schema is removed. Existing PostgreSQL
ABI2/3/4 wheel smoke artifacts remain identical on the new wheel.

This implements conditional SQL/cell-lineage components, not actual Ashlar source
admission or native guarded execution. All US-009-AC1–AC4 full acceptance rows,
original132 shared/native cases and broader supported domains remain open. The
native provider, source/caller/current authority, real route/action decisions,
second caller, UC time-travel/policy and closing-release tests above remain required.


## Conditional result-refinement candidate

CONTRACT-010 and TD-009 define a planned optional checker, not current native
support. US-009-AC3/AC4 stay open. Implement `current_query_refinement` Rust tests
and `tests/current-query-refinement/` cross-host cases before claiming this
candidate implemented. No original132 assertion or case ID changes.

| Layer | Required independent assertions |
| --- | --- |
| Source/binding | Actual CQP/parent and selected lowerer; exact four input digests plus the success report's exact complete-frame digest; stale/foreign/mixed inputs, duplicate JSON, unknown members and missing columns refuse before evaluation. |
| Input values | All complete eligible cells checked even behind FALSE/LIMIT0; exact signed 64-bit endpoints and canonical values, full Unicode/empty strings, duplicate scan keys, missing rows/count mismatch and conflicting shared columns. |
| Relational | Equality/Boolean predicates and lexicographic ORDER BY comparisons, literal/parameter values, inner/self joins, multiplicity and empty joins; independently authored expected tuples. |
| Aggregate | COUNT, DISTINCT, grouping, SUM beyond signed 64-bit, positive/negative cancellation, global-empty null and grouped-empty no row; each complete contributing occurrence retained. |
| Order/limit | Ascending/descending, hidden group/order fields, bag permutations, equal-key reorder, partial tie selection, duplicate-substitution refusal, omitted earlier keys and selected later keys; unordered limit valid submultisets. |
| Result/custody | Exact public values plus complete private tuples/bags; same public result with wrong contributor, duplicate or missing result, wrong scan order/key, integer rounding, null/absence/withholding confusion and unsupported ABI refuse. |
| Bounds | Exact and one-over frame, scan, population, column, tuple and join-product boundaries; no sample/prefix success, no private values/counts in coarse errors or success reports. |
| Embedding/backend | Actual Rust/Python CQP and validated PostgreSQL/Databricks binding paths; installed-wheel source/context custody; old registrations reject the new extension/cohort. |
| Native host | Actual complete independently captured populations/counts, source/actions/current authority, statement/parameter/result custody, snapshot retention and closing checks; pure frame success never qualifies these premises. |

Use existing managed/native executions only as retained correspondence evidence;
a fresh native acceptance claim needs a fresh original governed execution.
Independent review must assess tie nondeterminism, full-result cardinality and
source completeness separately; a successful decoder or mirrored computation
alone cannot qualify them.


The refinement countermodel deliberately forges population/count/result together.
A conditional match may be correct under those supplied data and still supplies
no native completeness or authority; only the genuine host/source gate can
refuse that world. Keep this distinct from declared-count mismatch refusals.
Use exact `(scan,path)` selections for raw-predicate/masked-projection positives
and same-Field remap mutants. Include hidden ORDER keys, multiple ordering columns
with mixed directions, invocation/cut/report reuse and private outcome/timing
boundaries. Old lowerers' relational inequalities remain unsupported.

Refinement expansion regressions must cover repeated large source cells across
projections and hidden order keys, including LIMIT 0. Test the inclusive
1,000,000 expected cell/order reference-slot bound and refusal above it; check
repeated public-value/contributor-key UTF-8 payload against the 16,777,216-byte
pre-limit bound. Successful small captured frames do not waive expanded-result
limits or imply whole-process memory/timing assurance.

## Explicit local Spark contributor candidate

Feature-scoped controls MUST establish cross-target binding refusal, unchanged
Databricks output, Spark-native builtin emission, no SUM-context downgrade and
same-original refinement acceptance/refusal with exact target artifact anchors.
An actual native Python child control MUST preserve parent identity and compare
a matching contributor frame while refusing a wrong public result and a foreign
target binding. A separately supervised tiny local Spark/Delta fixture MAY
qualify the selected compiler/host mechanics only; it cannot close native
security or Unity Catalog requirements.
