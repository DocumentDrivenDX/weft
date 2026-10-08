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
