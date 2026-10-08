# B-006: Ashlar native preparation

B-005 merged as PR #7 at `3ad557b76d6b8239d6033b993cd564bea48c2dd1`.
B-006 work continues on `codex/b006-ashlar-databricks` in the main Weft checkout.
This checkpoint establishes native prerequisites, not backend acceptance.

## Owned storage sources

Ashlar checkout inspected at `e3ab6648e0340c59a077459647eb2df11afe1a5a`.
The sibling is advancing independently; these hashes pin the consumed documents:

| Ashlar source | SHA-256 |
| --- | --- |
| CONTRACT-003-delta-graph-tables.md | ec73e77e2023d192ed3c6eb3916b395fa04f8f1695380667b206855fcf765632 |
| CONTRACT-004-publication-resolver.md | 52492ad626cf046d2ff595d5aa91c20d086561c1aba0df3b6b70338bbd08514b |
| delta-layout-v03.sql | ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e |

`ashlar-delta/0.3` remains a draft. Canonical objects and edges carry signed64
physical identities qualified by source and type; logical keys remain separate.
Property maps are exact JSON text keyed by producer property IDs. Typed serving
projections are explicit, rebuildable mappings with presence columns. Weft must
not infer a property's home from its scalar type or invent projection tables.
Parallel edges retain independent identities; ordinary Weft joins retain bags.
Ashlar's consumer graph count/distinct conventions do not replace SQL semantics.

One validated immutable publication supplies every consumed table UUID/version.
The host checks custody, policy, schema, retained files and all read dependencies;
the compiler emits exact version references and obligations without database IO.
No latest-head fallback or trust in stored validation flags is permitted.
Ashlar's UMF binding remains deferred. Candidate implementation will require
explicit supplied logical identities/mappings against these owned layouts;
production binding and runtime adoption are separate qualification work.

## Executed prerequisite evidence, 2026-10-08

Run `tests/ashlar-databricks/native-prerequisites.py` with the existing authenticated
`aidev-cus` SDK profile and `WEFT_ASHLAR_EVIDENCE_OUTPUT` set to an owned output
directory. Five read-only statements passed on existing warehouse
`2439e1f2e37ac563`; no tables or compute configuration changed.

[Summary](B-006-prerequisites/summary.json) and
[full statement responses](B-006-prerequisites/statements.jsonl) retain native
column metadata, submitted parameters, terminal outcomes and statement handles.
The engine reports `4.2.0` with a zero build hash; this alone does not identify
the Databricks warehouse release/channel or qualify a broader runtime profile.

- Exact equality distinguishes composed/decomposed Unicode and trailing spaces.
- Two maximum DECIMAL(28,2) inputs sum to exact widened decimal text.
- DECIMAL(38,0) SUM overflow fails with ARITHMETIC_OVERFLOW rather than NULL.
- A UInt64 JSON token is observed as exact text in this one probe; absent and
  JSON null both extract to SQL NULL, requiring separate presence/type guards.

These are independently authored engine probes. They do not execute emitted
compiler SQL, establish arbitrary JSON numeric lexical preservation, qualify
delegation/publication, or close US-004-AC1–AC4. Next add failing mapped corpus and
binding/refusal tests, implement the registered backend, then execute emitted SQL
with independent expected values and actual Python/browser parity.

## Binding admission checkpoint

`crates/weft-databricks` now admits an explicit candidate binding against the
pinned owner layout. Four Rust tests pass (`cargo test -p weft-databricks`), with
[source custody and scope](B-006-binding.json). Tests were added before the
implementation; the initial build failed because the binding module was absent.
This is a component gate, not a complete registered backend or native acceptance.

Bindings carry original model pins, exact record/property identities, source/type
selectors, schema revision, explicit property homes and one publication vector
of fully qualified table names, UUIDs and signed64 nonnegative versions. Unknown
members, including nested identity members, malformed IDs, stale pins, duplicate
mappings and unknown native column meanings refuse. Names are quoted component
by component; discriminator/property values will become parameter slots during
lowering. Existing unknown unmapped UMF content remains unchanged.

The initial native-column profiles follow the owner-described canonical and
example serving layouts. `node_type_a.group_value/group_present` and
`rank_value/rank_present` are explicit typed homes alongside `props_json`.
`edge_ab.score` is DOUBLE and cannot establish an exact decimal carrier. A native
BIGINT cannot represent UInt64. Other typed projections need an explicitly
registered owner layout; this checkpoint does not invent their columns.

Admission establishes structural/model agreement only. Hosts still must verify
physical schema, mapping correspondence, complete publication, effective policy,
retained snapshots and custody before execution/publication. The next step is
registered capability assessment and lowering with executable scalar integrity
checks before user predicates, then independent native corpus comparison. The
application-read extension and actual Python/browser/backend integration remain
required B-006 work.

## Registered scalar SQL checkpoint

The candidate now implements the shared registered Backend trait for the 0.1
relational plan. Scans, INNER JOIN bags, filters, exact equality, grouping,
required scalar projection and SUM lower to Databricks SQL. The core/frontend has
no Ashlar switch and the backend performs no database IO. Actual Rust registration
and the public compile envelope execute in the fixture CLI. Default compilation
still refuses candidate capabilities without explicit allowCandidate.

Discriminators, revisions, paths and source literals are ordered typed parameter
slots. Identifiers are separately backtick-quoted and Delta versions are validated
signed64 literals. String comparisons/grouping explicitly use UTF8_BINARY.
Numeric/boolean outputs travel as exact text with the logical decoder metadata.
Each selected scalar has a separate owner-wide integrity query at the same pinned
version, before user filters or joins. Missing/null, wrong native carrier, source
numeric domain, decimal scale and schema revision failures refuse publication.
JSON VARIANT DOUBLE/exponent carriers are unsupported rather than coerced. Payload
Unicode/duplicate-key validation and independent projection correspondence remain
explicit host obligations; native type labels alone cannot establish those facts.

SUM uses a guarded TRY_SUM: a nonempty aggregate with a null finite result raises
WFT-NUMERIC-DOMAIN, and an empty aggregate retains SQL NULL. This protects against
overflow-to-null independently of ANSI mode; ordinary casts still require the
admitted exact native domains and completed integrity checks. The source result
is mathematically exact or an error, not implicitly restricted to source precision.

[Source/binary custody](B-006-scalar-compiler.json) records eight passing Rust tests.
The owner DDL is retained exactly at `spec/upstream/ashlar-delta-v03.sql`. Native
fixtures use its canonical object and manifest CREATEs in the owned schema
`client_dev.weft_b006_20261008_scalar`; no shared tables or warehouse settings
changed. Intentional corrupt rows are synthetic refusal controls, not admitted
producer data. The fixture manifest is not proof of production authority.

- [Sales corpus](B-006-scalar-native/summary.json): ten cases; two exact/empty
  results and eight refusals before user-query submission. Independent Python
  integer equality and high-precision Decimal loops establish expected bags and
  totals. Parallel matching logical values contribute repeatedly, isolated
  customers remain legitimate, and composed/decomposed names form separate groups.
  The widened result `200000000000000000000000000.02` remains exact native STRING.
- [Global SUM](B-006-global-native/summary.json): two emitted-query cases; all
  orders contribute, including unmatched foreign keys, and empty input returns
  NULL. Logical nullable/scale/decoder metadata and native STRING metadata agree.
- [Aggregate pattern boundaries](B-006-sum-pattern/summary.json): native DECIMAL38
  overflow raises, empty input remains NULL and a large finite UInt64-derived sum
  remains exact. This is pattern evidence, not enumeration of unbounded groups.

Full native statement responses and requests/compiler artifacts sit next to the
summaries. The sales log preserves the initial plain-SUM baseline and the final
read-only rerun after guarded TRY_SUM; final captured artifacts use the guard.
Native fixtures were not replayed: the rerun verifies existing UUIDs and the same
immutable publication/version vector. A timeout retains its live handle; writes
are never blindly retried. All source expected values remain independent of SQL.

Remaining B-006 work includes native-column homes, the 0.2 application-read
extension, actual host obligation/policy/publication refusal execution and real
Python/browser composition parity. All capabilities remain candidate; the engine
version report does not qualify a Databricks warehouse release/channel. This
checkpoint does not close the four US-004 criteria or start B-007.

## Native typed homes and required-scalar application checkpoint

The [checkpoint custody](B-006-application-compiler.json) records eleven passing
Rust tests and the actual native work below. The 0.2 registered lowering shares
scalar access, exact slots, integrity and result decoding with 0.1. It adds
required-scalar whole entities in authored member order, exact bag COUNT,
aggregation, named parameters, ordering, limits and single/composite keyset
comparisons. Composite continuation expands lexicographically, preserving the
complete ordered key rather than comparing components independently.

[Typed-home native results](B-006-columns-native/summary.json) cover 48 cases:
16 valid/empty results and 32 refusals. Both STRING and BIGINT values use the
owner's node_type_a column/presence pairs, every JSON/column home combination,
and signed8/64 logical domains. Filters hide the deliberately corrupt rows;
owner-wide guards still reject range/type/presence/NUL failures before user SQL.
Native sums and metadata agree with independently authored Python integer groups.
The missing vocabulary declaration in the first synthetic model was corrected;
only reads resumed after validating existing fixture UUIDs/version custody.

[Required-scalar application results](B-006-application-native/summary.json)
cover 112 native cases across the same four homes and two widths. Bag self-join
counts retain every match; grouped/global/empty counts remain exact. Entity
pages retain field order and BOOLEAN text metadata. Single and composite cursors
exhaust the seven independent rows at LIMIT2, including repeated string prefixes,
Unicode/trailing-space differences and exact signed extremes. Every page checks
its complete authored key against the same pinned source before query submission.

[Key refusals](B-006-key-refusal/summary.json) cover eight false unique-name
assertions. Independent fixture names have one duplicated group; native key
checks return exactly one violation. No page query is submitted. This proves
native integrity refusal, not a live authorization or publisher authority.

[Unsigned column evidence](B-006-unsigned-columns/summary.json) covers seven
explicit synthetic scalar views of positive native BIGINT IDs. Logical
unsigned1/2 ranges reject the offending rows; widths3/8/32/63 return the exact
sum of values1..7; UInt64/BIGINT refuses at mapping admission before SQL. The
native column supports complete narrower unsigned domains with stored-data
checks; it cannot establish a full UInt64 carrier. These finite rows do not
qualify all boundaries in the admitted widths or infer business-key meaning.

COUNT and integrity counts now use guarded DECIMAL38 sums of one. MAX(1) detects
empty input without relying on signed64 COUNT overflow. SUM uses the same MAX
presence test; an overflowing nonempty aggregate raises while empty SUM remains
NULL. Scalar/global SUM and overflow-pattern native regressions were refreshed
against unchanged fixtures after this shared change.

The unsigned-subdomain enhancement changes admission only. Current compilation
reproduces all 180 retained native-tested request/SQL/parameter/result/obligation
responses exactly; [custody comparison](B-006-current-artifact-custody.json) pins
the current binary. New unsigned cases execute that binary natively. Native logs
retain earlier baselines; latest summaries and captured artifacts describe the
current semantic paths. New complete compile captures are compact JSONL files
beside the corresponding summaries; expected results stay in independent oracles.

These are owner-authorized candidate components. Optional/compound/related values,
actual host authorization/pin/publication refusal execution and native Python /
real browser composition remain required. No live delegation, accepted producer
binding, arbitrary native type or production/release qualification follows.

### Original local-module membership correction

A failing test exposed an extra same-module restriction in binding admission.
CONTRACT-001 permits declared references across supplied modules within one
owning document. Admission now follows the exact original member identity;
document/revision pin and declared membership checks remain. Twelve Rust tests
pass, and [actual native typed-column execution](B-006-cross-module-native/summary.json)
retains the field's `types` module identity and exact grouped values. The
[custody record](B-006-cross-module.json) pins the changed source and harness.
This does not fetch modules or infer cross-document dependencies.

### Python and browser embedding checkpoint — 2026-10-08

The optional `ashlar-databricks-candidate` feature now composes the same Rust
backend through `weft-runtime`, the Python ABI and browser WASM. Registration
is absent in the default build; a combined Ashlar/Truss candidate build passes
the request-level registration and candidate opt-out test.

`tests/ashlar-databricks/python-check.py` checks 188 saved native-tested full
compiler artifacts (including refusal responses) against the native Python
extension, with `subprocess.Popen` forbidden and PATH empty. All responses and
repeat calls match. Actual Chromium 148.0.7778.96 with Playwright 1.62.1 checks
the same 188 requests through the TypeScript wrapper and WASM, byte-for-byte
against Python. Runtime network APIs are disabled after bootstrap, no Node
globals are available, and the harness exercises transport errors and instance
retirement after a WASM trap. See B-006-python-summary.json and
B-006-browser-summary.json for binary hashes and runtime receipts.

This is embedding parity against previously recorded database-tested artifacts,
not new database execution or an independent result oracle. The initial
sandboxed Chromium launch failed before executing requests because macOS Mach
service registration was denied; the authorized unsandboxed launch passed.
`scripts/run-b006-embeddings.sh` rebuilds both embeddings and repeats the checks.
Optional/compound/relationship coverage and host enforcement remain unfinished;
this checkpoint does not complete B-006 or qualify a production backend.
