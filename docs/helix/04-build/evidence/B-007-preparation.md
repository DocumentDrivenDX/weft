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
