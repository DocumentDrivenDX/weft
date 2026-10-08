# B-007 preparation — 2026-10-08

State: in progress. B-006 merged as PR #8, commit
`1ed1c24dce356e1dfe34ae049959f4af52ea0bc5`, before this slice started.

The first component is a local native-backend support-claim verifier. It reads
hash-pinned reports, requires exact compiler/dialect/IR/backend/target/engine/
layout/model/binding/settings context, and compares independently supplied
expected rows. Explicit bag comparisons retain duplicate counts; ordered cases
require exact row order. A passing summary alone cannot replace semantic results.
Missing/failed/skipped evidence, duplicate/unknown report fields, missing layers,
invalid/stale hashes, rounding, normalization changes and absent/null substitution
refuse. Unqualified engines and candidate claims cannot be silently promoted.

`tests/qualify-and-evolve/evidence-check.py` passes 41 independently authored
controls. Missing implementation, ordering and malformed-digest failures precede
those implementations; retained logs and summary are in B-007-support-audit/.
These are synthetic audit-algorithm tests, not native qualification or a completed
support inventory. The verifier checks consistency/custody of trusted local test
reports; it cannot manufacture producer provenance or prove oracle independence.

The complete B-007 gates remain intact:

- Audit all 30 P0 criteria against actual layer-appropriate evidence.
- Evaluate all 636 initial fixtures and at least 1000 distinct expanded assertions.
- Execute at least 10000 deterministic property cases with pinned generator/library
  versions, recorded seeds and persisted minimized regressions.
- Exercise mutation/fuzz and critical supported/refused semantic branches.
- Qualify every claimed native operation/type/domain on its exact engine profile;
  retain candidate/unknown status where evidence does not prove support.
- Verify native Python and real browser behavior, interfaces, pins, unknown-content
  retention and resource/refusal boundaries.
- Publish a versioned support inventory and review packaging/release procedures,
  package ownership and license before distribution. The owner license choice is
  pending; this does not stop independent conformance work.

The first component closes none of those broader gates by itself. No package
publication, production profile or broader platform claim is made.

The second component passes 10000 deterministic frontend property cases using
pinned Proptest 1.11.0 and ChaCha seeds recorded in B-007-properties/summary.json.
Independent integer bounds and decimal coefficient arithmetic check acceptance
and exact literal retention; further properties check Unicode/opaque-content
retention, stale pins, selected unknown meaning, duplicate JSON keys and multiple
statements. All four properties pass, with no minimized failures to persist.
The configured failure persistence file is property-regressions.txt; future
failures shrink and persist there. Generator version is weft-properties/0.1.0.
[Proptest configuration](https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html)
defines successful-case counts and failure persistence. Repeated generated
requests are counted transparently: unique request digests are recorded separately
and do not stand in for the distinct expanded-assertion gate.

This closes the generated frontend-case count component only. Relational/bag
semantics, mutation/fuzz, full fixture and runtime matrices, support inventory
and release gates remain open. These receipts do not qualify a native engine.

The initial 636-case corpus now has a fresh frontend/oracle run. All cases
match their authored resolution/refusal expectations, including 333 refusals;
13 cases also compare independently interpreted relational bags using Python
integers and Decimal precision 100. Original modules and capability-set order
are checked for successful cases. B-007-initial-oracle/summary.json records
corpus, interpreter and freshly built frontend binary hashes, with a binary
custody check across the run. The oracle accepts explicit binary/output paths
for repeatable qualification runs while preserving its default B-002 command.
This is frontend evidence; the full backend/runtime matrix remains open.

The expanded frontend relational component passes 1200 distinct assertions
across 300 generated datasets (projection, boolean filter, equijoin, grouped
exact sum). Expected rows are calculated directly from supplied data without
using compiler IR; a separate IR interpreter must produce the same bags.
Unique entity keys coexist with repeated projected names and repeated order
rows, forcing duplicate/fanout preservation. Decimal inputs extend above the
binary-float exact integer range; Unicode names retain authored distinctions.
The fixed seed, Python version, harness/interpreter/binary hashes and every
input digest are retained in B-007-relational/. The initial 636-case oracle
was rerun after making its interpreter importable and still passes.

Output-corruption controls detect duplicate collapse in 600 cases and decimal
conversion through binary double in 600 cases. These are expressly not
compiler-source mutation evidence. This contributes expanded frontend
assertions; backend/embedding matrices, source mutation, fuzz/resource branch
audits and the final support/release inventory remain open.

The resource component passes three Rust tests, including 2000 deterministic
truncated/nested-duplicate input pairs and 4000 public-envelope atomic refusals.
It proves exact JSON node-count acceptance at 100000 total nodes and refusal
at 100001, plus oversized request (16 MiB + 1), SQL (64 KiB + 1) and binding
(4 MiB + 1) refusals. Nesting 200 refuses; no untested exact depth boundary
is claimed. Every named refusal exposes no SQL, parameters, logical plan,
result or host obligations. B-007-resources/ retains the log, seed, source hash
and explicit branch list. This bounded generator is not coverage-guided parser
fuzzing, and does not close plugin-failure or all critical-branch coverage.

Three actual compiler-source guard mutations are detected by existing tests:
disabling nested duplicate-key validation, disabling JSON node-count refusal,
and disabling original UMF module digest validation. Temporary copies alone
are mutated; the main checkout is untouched. B-007-source-mutations/ retains
exact substitutions, original/mutant hashes and failing test logs. The pin
mutation shrinks to (empty text, opaque zero); its persisted mutant-only seed
is retained as such, not counted as an actual compiler defect. These three
controls establish source-mutation sensitivity for those guards only. Required
backend type-filter, duplicate, rounding and absent/null source mutations remain
to be verified.

The source-mutation suite now also detects explicit native JSON null being
substituted with absence in PostgreSQL's registered presence observation.
The unchanged original-definition presence test passes first; the temporary
mutant fails its expected null refusal. Exact source hashes, replacement and
baseline/failure logs are retained with the four-mutant summary. This covers
the presence decoder mutation, not target SQL null behavior or the remaining
type-filter, duplicate-elimination and numeric-rounding source mutations.

Six source mutants now fail conformance tests. The two added SQL-emission
mutants remove the owner type predicate from a candidate scan and insert
DISTINCT into projection. An independently stated bag/owner-selection test
passes on the original compiler across fixture homes and fails on each mutant.
Baseline and mutant logs plus exact substitutions/hashes are retained. These
are structural emission-contract checks, not native mutated-result evidence;
numeric rounding mutation and native semantic mutation gates remain open.

Seven source mutants are detected after adding binary-double conversion of
original decimal operand tokens. The exact operand baseline passes; the mutant
changes 9999999999999999 to 10000000000000000 and fails the independently
authored exact-token assertion. Baseline/failure logs and original/mutant hashes
are retained. The four named semantic mutation classes (type filters, duplicate
elimination, rounding, absent/null) now have actual source mutations detected
at emission/token/decoder layers. This does not claim native mutant execution
or replace the release's native evidence and complete branch/runtime audits.

A fresh Python 3.12.14 host execution of all 463 saved Ashlar native-tested
compiler artifacts passes exact full-response parity, determinism, native
extension identity and subprocess/PATH-disabled checks. B-007-python/ records
the extension SHA-256 and log. This reuses the final hash-pinned B-006 extension
(8e04e6812141e895c8002624d0d81e9288dcc645d918855a2fecc5fd3546d831);
B-007 changes thus far affect tests, development dependencies and evidence,
not production compiler source. This is fresh host execution of saved artifacts,
not a new build or fresh native database execution. Broader runtime/release
qualification remains open.

A fresh real Chromium 148.0.7778.96 / Playwright 1.62.1 run passes all 463
Ashlar saved-artifact comparisons against native Python byte responses. It
reuses the final B-006 WASM SHA-256
73cf013ac6b14e07222f81201820cc049975c04fa93e3db8f37813e0712bc046.
The harness checks browser imports, no Node globals, runtime network refusal
and wrapper behavior. B-007-browser/ retains hashes, runtime versions, full
summary and log. The first sandbox browser launch terminated with SIGTRAP/EPERM;
the authorized host launch succeeded. No unavailable run is counted as passing.
This is fresh browser execution, not a new build or native database rerun.

Public plugin-failure boundaries now pass 26 combinations (13 fixture behaviors
across both compile/dialect versions). Lowering panic, missing mapping/capability
coverage, explicit lowering refusal and malformed output labels/types/carriers/
parameters/source/nullability all yield their expected diagnostic and no SQL,
parameters, logical plan, columns or obligations. B-007-plugin-boundary/ records
the passing log, source hash and named branches. Trusted synthetic plugins
exercise the actual Rust public envelope; this does not claim all plugin phases
or cross-runtime panic equivalence.

The seven-mutant suite was rerun with expected failure-signature checks. Each
mutant must fail its selected test at the intended semantic assertion; a generic
nonzero exit, build failure or unrelated panic cannot count as detection.
The summary now retains those signatures alongside substitutions/source hashes.
All seven pass this stricter detection audit.

The full workspace all-features/offline/locked run completes with 191 passing
tests and zero ignored. B-007-workspace/ retains its entire log and count summary.
Its source-at-launch scope excludes later public-plugin and parser additions,
which have separate receipts. The exhaustive original decimal property test
covers all 434 precision/scale pairs in both homes and passes.

A new deterministic Proptest SQL-text parser component passes 5000 cases:
1216 accepted, 3784 refused, with repeatable outcomes and in-range UTF-8
diagnostic spans. B-007-parser/ records the fixed seed, generator/library version,
source hash and log. The first command used a shared mutation target and executed
zero tests; that run is explicitly rejected and retained. A fresh target build
executes the named test and all 5000 cases. This is bounded generated parser
text testing, not coverage-guided fuzzing or proof of every syntax branch.

A freshly built test-third-only public runtime passes all 1273 authored compile
envelope cases. Seven compiled fixture queries independently execute in SQLite;
this is fixture backend SQL, never a substitute engine for either native target.
B-007-public-corpus/ retains compiler/corpus/harness hashes and build/run logs.
The reports harness now accepts explicit binary/output paths and checks binary
custody across execution. Native Python/browser rebuild parity for this corpus
is the next component.

A freshly built Maturin 1.9.6 test-third-only abi3 wheel passes all 1273 public
compile cases on native CPython 3.12.14, byte-identical to the fresh CLI reports.
PATH is empty and subprocess calls are forbidden; non-string arguments and
lone surrogates refuse. B-007-public-python/ records wheel/extension hashes,
actual runtime and build/run logs. The harness accepts explicit report/output
paths and hashes the loaded extension. This macOS ARM64 execution does not
qualify every Python version or architecture advertised by wheel metadata.

The fresh test-third WASM build now passes all 1273 public compile cases in
Chromium 148.0.7778.96 / Playwright 1.62.1 with exact CLI byte parity. The
real-browser harness also checks no Node globals or runtime network calls and
wrapper refusal/trap behavior. All 1273 fresh CLI responses validate against
their own versioned response schemas. B-007-public-browser/ retains build/run/
schema logs, actual runtime/import/memory reports and hashes for WASM, glue and
the reused unchanged B-004 wrapper. This completes the fresh public fixture
corpus CLI/Python/browser component; it is not target-engine qualification.

Case-level Ashlar application reconciliation passes all 112 historical native
application cases and 360 integrity receipts. Every emitted query/parameter
vector matches its native statement; terminal successful untruncated single-
chunk results match the recorded outcome and STRING column names/types. Each
emitted scalar/key guard has its own matching successful zero-violation receipt.
Empty native results legitimately have zero chunks and are handled explicitly.
B-007-ashlar-application-reconciliation/ pins input reports and authored native
harness source and lists case/query/statement identities. This establishes saved
receipt consistency/custody, not fresh native execution or a new independent
result oracle. Application story reconciliation remains incomplete across
relationships, compounds, Truss and host/publication boundaries.

The 48 compound entity/keyset cases reconcile against the accepted frozen-run
receipt set (240 statements, 192 integrity guards). The accepted-file checksum
and custody count must match; every receipt is consumed exactly once. Actual
native SQL/parameters and terminal untruncated rows match the artifacts. Strict
decoding rejects duplicate JSON members/nonfinite constants and compares exact
authored values, distinguishing Boolean from numeric JSON values. Empty second
pages match their explicit expectations. B-007-ashlar-compound-page-reconciliation/
records all case/statement identities and source hashes. Earlier rejected mutable-
binary attempts are not used. This audits saved evidence, not a new execution.

Relationship reconciliation passes 52 saved native cases: 28 exact duplicate-
preserving bags and 24 integrity refusals before the user query. All 272 unique
reused guard labels match emitted SQL and only its used parameter values;
recorded counts agree with actual successful untruncated receipts. Forward/inverse
bounded key lists and EXISTS results match their authored expectations without
collapsing duplicate tuples. Repeated labels from earlier attempts are accepted
only when every referenced receipt has identical SQL/parameters/terminal data;
all corresponding native statement IDs are retained. No latest-attempt assumption
or ambiguous label silently substitutes a different result. This is saved
evidence reconciliation, not a fresh run or production relationship claim.

A fresh candidate Truss application run passes all 76 actual PostgreSQL cases
(37 props, 39 row homes) using the existing isolated PostgreSQL 17.9 aarch64
fixture container with UTF8 encoding. The newly built compiler is copied outside
Cargo targets and frozen; its digest is checked across execution. Independently
authored expectations cover application bags/entities/counts/relationships/pages
and corruption controls in the native harness. B-007-truss-application-native/
retains build/run/summary records and losslessly compressed full compile/native
reports with both compressed/uncompressed custody hashes. These are owned
temporary-table fixtures, not installed Truss or production authority evidence.

Fresh Truss-enabled native Python and browser WASM builds match all 76 newly
PostgreSQL-tested application compiler artifacts byte-for-byte. CPython 3.12.14
uses the actual ABI extension with subprocess/PATH disabled; Chromium
148.0.7778.96 / Playwright 1.62.1 exercises the real WASM/wrapper.
B-007-truss-embeddings/ records build features, wheel/extension/WASM hashes and
full logs. The earlier Ashlar-only wheel correctly refused this unregistered
Truss backend; its failed parity attempt is retained, not counted as passing.
This closes this candidate fixture corpus's fresh CLI/Python/browser parity
component, not all possible original-definition configurations or platforms.

The combined saved-evidence replay now includes the fresh Truss archive: six
components pass and 24 referenced evidence hashes verify. The archive check
validates compressed/uncompressed hashes and byte count, all 76 unique corpus
IDs, raw/parsed artifact equality, original module/binding pins, lexical parameter
carriers, native result row arity and matching Python/browser parity counts.
This is archive consistency, not an additional native run or independent oracle.

Explicit parser-limit boundary review now has 24 passing assertions across both
dialects: output 256/257, joins 16/17, SQL byte 65536/65537 and token 4096/4097.
Application bounds accept 1/1000 and reject zero, above-limit, negative, decimal,
string and u16-overflow forms. Token-at-limit malformed syntax reaches ordinary
syntax refusal; token-over-limit reaches WFT-LIMIT. B-007-parser-boundaries/
records the named test execution, source hash and branch values. These parser
checks do not imply successful model resolution or backend qualification for
every maximum-sized query. Broader critical model/backend branches remain open.

The complete application-model file passes 11 tests. The added resource test
proves selected descriptor graph depth 128/129 and identities 4096/4097 acceptance/
refusal boundaries. Existing tests cover authored member order, absent-allowed
availability without SQL-nullability coercion, recursive identity graphs, exact
list item types, required scalar keys, authored relationship/inverse identity,
ambiguity/unkeyed inverse and unsupported selected meanings. Unrelated unknown
members do not block a known projection; selecting their descriptor refuses.
B-007-model-boundaries/ records all executed names, source/implementation hashes
and log. Descriptor admission at a boundary does not imply that an expanded
entity query exceeds neither its separate output limit nor native decoder limits.

Eight unsigned-BIGINT boundary controls now pass against the actual existing
Databricks warehouse, with 22 terminal statement receipts. Widths 1/2/3/8/16/32/63
accept [0, 1, maximum] and exactly sum to 2^bits (including UInt63's result
9223372036854775808); negative and representable above-domain values trigger
emitted integrity guards. UInt63's physically unrepresentable upper neighbor
raises native CAST_OVERFLOW; UInt64/BIGINT mapping refuses compilation before
SQL. The frozen CLI checksum is unchanged. B-007-unsigned-boundaries-native/
retains requests/artifacts and all native success/expected-error receipts.
These read-only controls explicitly replace the physical owner relation with
synthetic rows; they exercise emitted native operations, not table publication
custody, every integer width, or production warehouse qualification.

The acceptance evidence index now references the unsigned native boundary,
backend branch, application-model graph and parser boundary receipts only under
relevant criteria. Assessments remain unclosed. Replaying retained evidence with
CPython 3.12.14 passes six components and verifies 28 evidence references. An
initial invocation with the system Python failed because its `zip` lacks the
`strict` argument; that invocation is not passing evidence. The configured
Python environment completed the replay; no new database execution is implied.

A retained unsigned-boundary reconciler independently checks all eight authored
width controls against pinned module/binding bytes, emitted SQL substitutions,
exact parameter arrays and integer-computed expectations. All 22 distinct native
statement IDs are consumed exactly once: 21 successful single-row STRING results
and one expected terminal CAST_OVERFLOW failure. UInt64 remains a compile refusal.
This checks saved synthetic receipts; it adds no database execution or production
publication claim. The combined retained-evidence command now runs seven components.

Unsigned receipt corruption controls accept the untouched baseline and reject
14 temporary changes: SQL, parameters, rounded UInt63 sum, zeroed domain guard,
wrong overflow terminal state/error, truncation, row count, carrier, duplicate
labels/statement IDs, module/binding pins and an accepted UInt64 response. Each
refusal must be an assertion failure rather than an unrelated execution error.
The eight-component retained replay includes these controls; originals are unchanged.

Local package inspection verifies two tested Python wheels' RECORD hashes, sizes,
unique safe paths, Mach-O extension hashes and native ABI/platform tags. Both
match the actual extensions used by their B-007 parity runs. Neither includes
license metadata. The browser package remains private and exports TypeScript
source rather than built distribution files. Exact findings are retained under
B-007-package-inspection/; release closure remains unproven.

The browser transport now builds with pinned TypeScript 5.9.3, emitting JavaScript
and declarations; package exports target those built files. A fresh Chromium run
using the newly emitted transport passes all 1,273 public test-third corpus cases
with exact native-response parity. The retained WASM compiler is unchanged.
B-007-built-browser/ pins generated output, configuration, manifest and lockfile
hashes. This is the transport package build, not a bundled compiler/backend release.

The built browser package also passes an isolated packed-archive consumer check:
ESM resolution by package name, scalar/trap controls and strict declaration use.
The four archive members and exact bytes are hashed in
B-007-browser-package-consumer/. This supplements actual-browser compiler parity;
it does not replace it or qualify a bundled backend/registry release.

Seven independently specified raw malicious/resource inputs now pass atomic
refusal assertions through the candidate CLI, native Python and real Chromium
WASM: nested duplicate keys, truncated JSON, 100001 JSON nodes, exactly 16 MiB
malformed whitespace, 16 MiB plus one byte, SQL over 64 KiB and binding over 4 MiB.
All host responses match byte-for-byte; the at-limit malformed request is INPUT,
not LIMIT. Python disables subprocess/PATH after CLI receipt generation; browser
compilation I/O is disabled. B-007-host-resources/ retains exact input hashes,
byte counts, diagnostics and executed binary/extension/WASM/wrapper hashes.
These are early common-boundary checks across deliberately named feature builds;
they do not qualify every host resource path or native database behavior.

The retained-source audit inspects the actual preparation responses, rather than
inferring retention from host parity. All 303 resolved cases preserve supplied
module arrays exactly, including documentJson strings and ordered pins; 304
retained document instances include opaque root extension content. All 333
refusals have no logical plan. B-007-source-retention-audit/ records input hashes
and the separate public envelope distribution: seven compiled fixture successes
and 1266 authored refusals, including 608 BACKEND-MISSING cases. The 1273-case
host parity count is consequently a public success/refusal boundary matrix, not
1273 native target executions. Native backend evidence remains separately scoped.

US-001's retained-plan audit explicitly checks customer/orders source identities,
the order-total aggregate identity, ambiguous-source refusal and selection of the
other-sales document by qualification. The independent Decimal/bag interpreter
replays all 13 authored row expectations, including exact large totals, string
filters and empty global/grouped aggregates. All 333 authored refusals match their
diagnostic codes and contain no plan/SQL. B-007-resolution-audit/ records every
case ID plus report/oracle hashes. This is saved-plan semantic reconciliation,
not another compiler or native-engine run. The 1200 generated relational assertions
remain separate evidence for broader deterministic bags.

US-002 registration audit runs all 12 backend-registry, backend-pipeline and public
compile-envelope tests with the locked offline toolchain; none are ignored or
filtered. Named tests and source hashes are retained in B-007-registration-audit/.
Both plan versions use the synthetic third backend. Source review confirms keyed
Registry lookup and typed registered lowering, with no storage-backend selection
branch in the frontend. Candidate opt-in/upgrade, version/capability failures,
hostile mappings and atomic plugin/emission failures have positive/refusal cases.
These prove the trusted registration boundary, not native backend semantics or
independent provenance for a plugin's declared evidence strings.

Application acceptance review found missing explicit Databricks empty grouped
COUNT evidence. A fresh native component now executes all eight signed8/64 ×
props/column home combinations on the retained synthetic snapshots. All eight
prequery scalar-integrity guards return zero and all grouped COUNT queries return
no rows: 16 successful native terminal statements. B-007-empty-grouped-count-native/
retains exact compile inputs/artifacts, native SQL/parameters/terminal responses,
outcomes, frozen compiler and harness hashes. The initial wrong artifact-file
lookup failed before statement submission; the corrected run uses the retained
JSONL corpus. No data write or production qualification is performed.

The full locked offline all-features workspace run is terminal success: 195 tests
across 35 suites, none failed/ignored. B-007-final-workspace/ records the exact
pre-addition Rust source hashes and lockfile. After it finished, a separate new
Catalog preparation test passed seven explicit assertions: empty/32/33 documents,
256/257 selected modules and 4194304/4194305 document bytes. The at-boundary valid
inputs are admitted; adjacent limits refuse. Its named-test log and source hash
are in B-007-model-input-boundaries/. Do not combine these into a claimed fresh
196-test workspace run. The named critical-path review now references the
executed module limits and terminal checkpoint; native-profile and owner release
qualification remain independent open gates.

Catalog::prepare now has explicit guard accounting: one valid baseline and 13
refusal assertions check both code and intended diagnostic message, supplementing
the seven document/selection/byte boundaries. All 17 explicit refusal conditions
(including each combined-condition operand) have a named assertion. The fresh
filtered test executes exactly once and passes; B-007-catalog-branches/ pins test
and implementation sources. This is one function's branch accounting, not a
blanket claim about every compiler/schema/JSON function. No compiler behavior changed.

Public request admission adds 11 explicit branch assertions with exact diagnostic
messages, response interface versions and absence of SQL/parameters/plan/columns/
obligations. Missing/non-string versions, mismatched/unknown version pairs, both
versioned-envelope failures and correctly hashed malformed/duplicate-key binding
JSON all refuse at their intended phase. B-007-request-branches/ retains the
executed named test and test/implementation hashes. Other request-resource/hash/
model/lowering paths remain separately accounted; no all-function claim follows.

Trusted host registry factory branches pass eight explicit cases across both
compile interfaces. A returned error and an empty registry refuse without falling
back to the compiler's deliberately working default registry. A successful factory
receives pinned catalog/typed plan/binding context and supplies the emitted backend.
A stale binding digest refuses before the factory is called. The named test passes
once; exact hashes/log are in B-007-factory-branches/. Arbitrary trusted host callback
panic containment is not claimed by these error-return tests.

The JSON inspector has 25 explicit value/error assertions: ten accepted values
include each primitive/container branch and an integer beyond machine ranges;
eleven malformed/trailing/nonfinite spellings refuse; four duplicate cases cover
raw keys, Unicode-escaped equivalent keys and nested array/object propagation.
The fresh named test passes and its implementation/test hashes are retained in
B-007-json-branches/. Exact node/byte limits remain separately evidenced; no new
claim about every recursion-depth boundary is made.

Registry description guards now have a fresh explicit test for describe panic,
returned refusal and invalid manifest interface. Each failure leaves no registered
identity; recovery by valid registration succeeds; duplicate registration refuses
without replacing the original manifest. The fixture forbids binding/assessment/
lowering/emission during registration. B-007-registration-description/ retains
source hashes and the executed single-test log. Compiler production code is unchanged.

Ten explicit v0.1 adapter dispatch refusals pass with exact diagnostic messages:
backend version, IR version, duplicate operation identities, unknown target profile,
binding profile, stale plan/catalog pins, oversized binding, stale digest and
correctly hashed malformed/duplicate-key binding JSON. They reach the intended
pre-lowering guards. B-007-dispatch-branches/ retains the fresh named test and
source hashes. V0.2 positive composition remains separately tested; later
capability/mapping/emission branches are not declared complete by this component.

Eight assessment guard cases pass: duplicate/over4096 additional capabilities,
duplicate/unrequested assessments, unsupported assessment and duplicate/undeclared/
missing supported evidence. A fixture panic forbids reaching lowering/emission;
exact capability diagnostics demonstrate each intended earlier guard. The initial
fixture borrow-check error was corrected before successful test execution and is
not semantic evidence. B-007-assessment-branches/ retains the passed named-test
log and source hashes. Existing candidate opt-in/upgrade and missing-assessment
checks remain separate.

Nine obligation merge cases pass: six malformed ID/parameter/code forms and two
same-ID parameter/owner conflicts refuse with the intended obligation diagnostic.
Identical requirements deduplicate and distinct requirements survive sorted in
the emission. B-007-obligation-branches/ retains source hashes and executed test.
An initial fixture move-check error was corrected before successful execution;
compiler implementation remains unchanged. Actual host fulfillment is separately
proved by native driver controls, not by these merge assertions.

The expanded combined guard regression passes 36 tests across five suites,
with none ignored or filtered. B-007-guard-regression-expanded/ retains the
terminal log and current test-source hashes. US-002 matrix entries now link
dispatch, capability-assessment, and obligation-merge branch receipts. This
component checkpoint does not replace full-workspace or native qualification.

B-007-language-root-branches/ retains 14 exact refusal assertions across
both plan versions: an unaccepted dialect/IR pair and six non-object binding
roots each. A panicking backend fixture proves refusal precedes validation.
The fresh named test passes; this does not establish all dispatch branches.

B-007-coverage-branches/ records eight executed coverage refusals for missing
records/fields and revision mismatches in both plan versions. Panicking later
phases detect accidental assessment, lowering, or emission. The fixture does
not select named types or relationships; those coverage arms remain separate
audit work. The initial overbroad type-fixture assertion was corrected.

B-007-type-relationship-coverage/ retains four passing refusal assertions
using resolved whole-entity and related-page corpus fixtures. Missing type or
relationship coverage and revision mismatches refuse before assessment. With
B-007-coverage-branches, all four coverage identity classes have explicit
missing-identity and revision-mismatch assertions; release closure remains open.

B-007-declaration-branches/ retains eight exact refusals across both plans:
missing operation declaration, excluded selected target, excluded selected
language, and declaration-side Unsupported status despite supported assessment.
The named test passes and downstream panic fixtures detect guard bypass.

B-007-cross-phase-obligations/ records six passing cases across both plans.
Conflicting declaration/assessment requirements refuse before lowering;
conflicting emitted requirements refuse before artifact return. Identical
requirements from all three phases survive once in each compiled artifact.

B-007-emission-branches/ retains sixteen passing emission guard refusals
across both plans: SQL content/size, parameter count/origin/nullability, and
column position/source provenance. Exact WFT-EMIT messages identify each
guard family. Remaining representation and numeric-domain branches are not
claimed by this checkpoint.

B-007-parameter-domain-branches/ retains twenty refusal and six accepted
parameter validation assertions. Exact diagnostics cover lexical/domain
guards; signed 8-bit endpoints, decimal upper bound, canonical booleans, and
Unicode pass. This named unit test does not claim target execution.

B-007-related-representation/ retains an accepted relationship metadata
baseline and seven passing corruption refusals: revision, bound, key identity,
fields, types, nullability, and representation kind. The test derives authored
key metadata from the resolved application fixture; no native execution claim.

B-007-field-representation/ records the expanded passing presence/native-null
test with three new descriptor/nullability/scalar-type refusals. The fixture
correction removes an invalid paging-key assumption. Metadata validation
evidence is scoped to this resolved fixture, not target execution.

PR #9 remains open/draft at the current pushed source. Its description now
reports the 195-test historical workspace checkpoint and later individually
scoped guard/representation evidence, real native counts, local packaging,
and unresolved native/support/release gates. The acceptance matrix links the
new relationship and field representation receipts without promoting status.

B-007-core-unit-current/ retains seven passing current core unit tests with
none ignored or filtered. The new 0.1 projection-root corruption refuses with
the exact intended diagnostic before output validation. This incorporates
current field and relationship representation tests, not full workspace or
engine qualification.

B-007-native-profile-scopes/ groups the 76 actual retained Truss artifacts
by exact compiler/dialect/IR/backend/model/binding pins: 21 distinct compiled
scopes. The executable audit checks raw/decoded response agreement and unique
case IDs. Engine/session provenance, expectations and host layers remain
separate joins before a real support report; no inventory promotion occurs.

The native profile audit now reconciles all 76 saved Truss row sets with
independently authored harness expectations using exact Decimal/value
canonicalization, including 65 ordered comparisons. Original request
model and binding digests are verified per case. row-reconciliation.json
retains this expanded observation separately from the original scope-only
receipt. Only expectation definitions execute; no compiler or database rerun.

The saved native scope/row audit now removes compiler-path/hash setup from
its trusted expectation-definition prefix, eliminating the temporary binary
dependency. portable-reconciliation.json retains the passed 76-row audit.
Eight executable corruption controls reject row loss/duplication, altered
numbers/pins/order, duplicate case IDs, and raw/decoded disagreement.

The standard retained-evidence runner now executes ten components, including
the portable native profile/independent-row audit and its eight corruption
controls. It verifies 65 referenced evidence hashes. US-003/US-006 matrix
entries link these receipts. This replay does not promote candidate profiles
or replace engine/session provenance and host qualification.

B-007-real-native-report-audit/ passes the support-report consistency verifier
on all 76 actual retained Truss cases across 21 pinned scopes, using independent
expected rows and exact normalized base-ten tuples. Layout revision is the
retained owner pin-manifest digest. Internal supported claims exercise the
verifier; they do not promote candidate inventory, producer trust, host-layer
qualification, or engine/session provenance.

The standard retained-evidence replay now runs eleven components and
verifies 66 evidence references, including real Truss report consistency
for 21 scopes/76 cases. The closure audit distinguishes this executed
consistency proof from still-unproved final provenance, required host layers,
and candidate-to-supported qualification.

B-007-truss-python-case-receipts/ retains a fresh 76-case native Python
embedding replay with subprocess entry points disabled. Every case records
request, actual-response and expected-response hashes tied to the loaded
abi3 extension hash. The existing wheel executes; PostgreSQL is not rerun.
This closes the per-case Python receipt-index gap, not browser provenance.

B-007-truss-browser-case-receipts/ records fresh actual Chromium 148
execution for all 76 Truss requests with exact artifact byte parity. Each
request/actual/expected hash is retained with the loaded WASM hash and
network/import/trap checks. PostgreSQL was not rerun. This supplies a
case-level browser receipt index for joining native report scopes.

The real Truss report audit now joins all 76 Python and 76 browser case
receipts to original request hashes and exact native compiler artifact hashes.
host-joined-summary.json retains loaded runtime identities and receipt hashes
for these 152 joins. They prove host compiler artifact parity, not separate
host database executions or final native/support provenance qualification.

Ten host-receipt corruption controls pass: each Python/browser join rejects
missing cases, duplicate IDs, changed actual/request hashes, and false parity
flags. The standard retained-evidence runner includes these controls. This
checks receipt consistency rather than reexecuting hosts or native engines.

Host joins now compare extension/WASM digest and runtime versions to the
independent retained embedding summaries. Four additional altered-runtime
controls refuse, bringing host receipt corruption controls to fourteen.
The full twelve-component retained-evidence replay passes. Identity
consistency does not assert producer trust or final support qualification.

The acceptance matrix now links case-level Python/browser receipts, their
real native report joins, runtime-identity corruption controls, and current
core unit evidence. Twelve replay components pass with 71 verified references.
The closure audit records verified compiler artifact parity while retaining
final profile provenance and host execution qualification as open.
