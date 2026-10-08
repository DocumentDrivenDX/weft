# B-007 critical semantic path review

The review follows public input → original UMF preparation → typed resolution →
registered lowering → exact artifact → host execution/publication. Evidence is
named by behavior, not a line-coverage percentage. Domain combinations beyond
those actually observed remain unqualified.

| Path | Positive/refusal evidence | Review disposition |
| --- | --- | --- |
| Raw JSON/request admission | Exact node/request/SQL/binding limits; nested duplicate/truncated JSON generators; seven actual CLI/Python/Chromium resource cases | Observed common boundaries agree and refusals omit partial artifacts. |
| Interface/profile selection | Strict public envelope, mismatched compile/dialect/version and unknown-key cases; explicit 0.1/0.2 syntax tests | No implicit version migration. |
| Original module admission | Source-retention audit: 303 resolved cases retain 304 exact documents; stale hashes/identity/version/selected unknown refusals | Original bytes/pins remain authoritative. Seven explicit document-count/selection-count/byte boundaries and 14 admission assertions account for all 17 explicit Catalog::prepare refusal conditions. |
| Names and scalar domains | US-001 exact scan/aggregate/qualified identities, ambiguity and 333 refusal cases; 10000 integer/decimal/Unicode/hostile properties | Exact independent arithmetic and selection; no first-match or float fallback. |
| Application type graph/keys | Ordered members, recursive identity graph, authored key scalar requirements, inverse/ambiguous relationship controls; depth128/129 and identities4096/4097 | Supported graph meaning and refused unknown selected meaning have named checks. |
| Relational bags/exact aggregates | 13 independent expected bags, 1200 generated relational assertions; native Truss/Ashlar corpora and grouped-empty COUNT supplement | Output corruption and actual DISTINCT/rounding mutations are detected; engines remain separately qualified. |
| Backend binding/capability/emission | US-002 12 fresh tests, 18 Ashlar/22 Truss branches, 26 public plugin failures; native/backend review | Candidate cannot upgrade; selected hostile/unknown mapping or invalid output refuses atomically. |
| Exact embedding transport | Actual native extensions/WASM, full byte parity and explicit wide integer/nullable SUM/text-slot metadata; trap retirement | Wrappers delegate to shared Rust; observed hosts only. |
| Driver custody/publication | Actual B-005/B-006 buffered drivers and prequery/prepublication failures | Native transport/guards proved in synthetic fixtures; production policy authority remains unqualified. |
| Evidence/support promotion | 41 verifier controls, receipt reconciliation/corruptions, exact hashes and candidate inventory | Failed/skipped/missing inputs cannot count as supported; fixture provenance cannot be manufactured. |

Actual source mutations detect removed type filtering, duplicate elimination,
rounding and absent/null substitution, as well as pin/JSON/node guard removal.
Deterministic parser text and JSON resource exercises retain generators/seeds;
no failure occurred to minimize. Unsupported grammar/profile/domain remains a
whole-operation refusal. The named module-boundary
check passes separately after a terminal 195-test, 35-suite workspace checkpoint. Final native-profile
qualification and owner release decisions remain independent gates.


## Registered adapter guard accounting

The current backend-pipeline integration suite passes all 12 tests, with none
ignored or filtered (B-007-pipeline-consolidated/). The following receipts cover
individual refusal guards; they supplement the earlier broad US-002 review.

| Guard | Exact assertion evidence |
| --- | --- |
| Manifest identities/evidence/language declarations; target count 256/257, capability count 4096/4097 and raw JSON one-MiB boundary | B-007-manifest-admission-boundaries: 17 refusals and three admitted boundaries; seven registry tests pass |
| Immutable registered manifest, describe-once and version/settings retention after backing declaration changes | B-007-manifest-snapshot: both IR versions, changed-version refusal, sixteen pipeline tests pass |
| Backend version, actual IR, duplicate operation IDs, target profile, binding profile, model pins, binding size/digest/JSON | B-007-dispatch-branches |
| Each target identity field rejects empty/NUL values; session settings reject non-object shapes | B-007-target-profile-guards: fifteen explicit refusals with exact diagnostics and positive retained-version/settings assertions |
| Manifest dialect/IR acceptance and non-object JSON roots, both plans | B-007-language-root-branches |
| Missing or wrong-revision record and field coverage, both plans | B-007-coverage-branches |
| Missing or wrong-revision type and relationship coverage, resolved application plans | B-007-type-relationship-coverage |
| Non-object/empty domains, malformed/duplicate/undeclared evidence and constraints, target/language declarations and malformed/duplicate obligations | B-007-capability-declaration-guards: 24 capability and nine obligation refusals, exact diagnostics, valid obligation baseline |
| Duplicate/oversize additional capabilities; duplicate/unrequested/unsupported assessments; duplicate/undeclared/missing evidence | B-007-assessment-branches |
| Malformed obligations, same-ID conflicting parameters/owner, identical deduplication and sorted distinct retention | B-007-obligation-branches |

Coverage selection is asserted nonempty in each tested identity class; the
panicking downstream fixtures establish the intended refusal phase. This does
not establish every profile combination. Declaration scope/status and cross-phase obligation conflicts now have
individual assertions (B-007-declaration-branches and
B-007-cross-phase-obligations). Emission bounds, slot metadata, provenance,
and exact parameter-domain guards have explicit assertions in
B-007-emission-branches and B-007-parameter-domain-branches. Remaining
representation guards require separate accounting; this is not a blanket
all-branches claim. Native support and release ownership are unchanged.


The latest consolidated checkpoint passes 15 pipeline integration tests and
four emission unit tests, none ignored; one unrelated library unit test is
filtered by the emission group selector. B-007-adapter-emission-consolidated/
retains both logs and current source hashes. Historical receipts are retained
as observations at their source revisions rather than relabeled as current.


## Emission validation assertion map

All eight current core library unit tests pass with none ignored or filtered
(B-007-core-unit-expanded/). Individual guard evidence is mapped below.

| Guard family | Assertion evidence |
| --- | --- |
| Empty/oversized/NUL SQL and excess parameters | B-007-emission-branches |
| Slot position, object origin, non-null parameter type | Existing pipeline emitter corruption test and B-007-emission-branches |
| String NUL, canonical boolean, numeric lexical text, integer width, decimal domain and exact range | B-007-parameter-domain-branches; original exact numeric slots test |
| 0.1 projection root | B-007-core-unit-current/projection-root.log |
| Output count/name/order/source identity | Existing pipeline emitter corruption test and B-007-emission-branches |
| Scalar family/carrier/decoder/facets/nullability | B-007-scalar-representation and original numeric result test |
| Projected field descriptor, Value presence/nullability and native-null qualification | B-007-field-representation; expanded presence/native-null unit test; B-007-native-null-assessments explicitly covers supported/candidate/unsupported and unrelated assessment IDs |
| Relationship identity/bound/key ID/fields/types/nullability/representation | B-007-related-representation |

This map identifies executed validation guards. It does not establish every
possible composition, every subcondition of every type descriptor, or native
backend lowering coverage. The broader P0 branch accounting and exact support
profile qualification remain open. Older full-workspace source hashes remain
historical evidence and are not replaced by this unit checkpoint.

The [manifest admission accounting](B-007-manifest-branch-accounting.md) reviews
each declaration guard family against the seven executed registry tests and
records their three isolated admission gaps closed by exact assertions in the
eight-test registry receipt. This does not close the broader semantic branch gate.

The positive mapping-derived requirement path now has explicit assertions in
`mapping_derived_capabilities_are_qualified_and_deduplicate_plan_requirements`.
Both IR versions preserve the added declaration, its domain/constraints/evidence
and supported assessment; a requirement overlapping the logical plan appears
exactly once. Emitted SQL remains the independently fixed fixture query.
B-007-derived-capabilities retains the current 17-test pipeline checkpoint.
This fixture evidence does not qualify either native backend.

The 0.1 scalar resolver now has its own semantic family accounting in
[B-007-scalar-resolver-branch-accounting.md](B-007-scalar-resolver-branch-accounting.md).
Two internal guards bypassed by the parsed-SQL corpus have direct assertions: the
1,024/1,025 literal boundary and empty JOIN predicates. Nine library tests pass,
none filtered or ignored. Application resolution, catalog/exact/parser and native
lowering remain separate scopes in the broader critical branch gate.

The application resolver now has its own semantic-family assertion map in
[B-007-application-resolver-branch-accounting.md](B-007-application-resolver-branch-accounting.md).
Nine tests pass without filtering, including the unchanged 620-case application
corpus and the 303 initial positive queries in explicit IR 0.2. Added assertions
cover named-parameter, expanded-entity, scan-sensitive grouping and read-profile
recognizer gaps. Parser/exact/type-graph and backend lowering accounting remain
separate; this scoped completion does not itself qualify native support.

## Selected graph and endpoint admission checkpoint

B-007-typegraph-audit retains all 15 application-model integration tests passing,
with no ignored or filtered cases. New checks assert missing ordered members,
non-Field members, missing selected names, container item role, structured target
role and scalar/reference ambiguity, missing/unowned member selection and
by-name/by-identity equivalence. Relationship checks cover polymorphic endpoints,
non-Record target, inconsistent source multiplicity, source-key fallback and
ambiguity, and non-member key fields.

Duplicate member/key references, scalar array shapes and facets on containers
are rejected by the pinned UMF envelope before semantic graph construction.
Tests assert that exact admission stage without claiming execution of downstream
defensive guards. Existing recursion, unknown meaning and exact depth/identity
boundaries remain in this unfiltered suite. Exact-value, parser and adapter
family review and native supported-profile qualification remain open.

## Exact-value and parser family review

[B-007 exact/parser accounting](B-007-exact-parser-branch-accounting.md) maps
eight exact-value and twelve lexer/parser semantic families to explicit tests
and scoped historical property/resource receipts. Two new exact tests pass
with five property tests intentionally filtered; all six application syntax
tests pass unfiltered. No production source changed. Backend lowering review
and supported native-profile qualification remain open.

## Every Ashlar scalar decimal domain

B-007-decimal-domains-native now records all 434 precision 1..28 and scale
0..precision pairs passing on Databricks SQL 2026.39 with the retained exact
u/r build hashes. There are 1,302 assertions in 55 batched native queries plus
three read-only engine/ANSI probes: 58 terminal successful statements. Per-domain
checks admit repeated positive/negative extrema and the least positive coefficient,
produce its exact SUM, and detect both precision overflow directions, excess
scale, native null and absence. Engine identity accompanies every assertion in
the same native statement. Separate ANSI probes observe true; they do not
attest settings for arbitrary hosts.

The fresh compiler is hash-pinned, with source/build custody. Independent
reconciliation reconstructs exact SQL, parameters, model/binding pins, result
scale and expected integer-coefficient arithmetic. Seven corruption controls
refuse after valid archive/custody rehashing. Compressed archives preserve the
original 11.6 MB artifacts/receipts in about 350 KB of retained evidence.
These are read-only synthetic owner substitutions for scalar props storage.
They qualify observed domain operations, not stored-table/publication custody
or a supported backend registration. The cancellation dataset alone does not
detect DISTINCT; separate retained duplicate-sensitive native/source-mutation
checks own that guarantee. No typed decimal home is invented for Ashlar's layout.

The exact/parser semantic-family review is now recorded separately; backend
lowering and final graph accounting, supported native registrations and the
final acceptance audit remain required.

## Final graph and backend family accounting

The graph ledger now maps 14 semantic families to all 18 passing application-model
tests. The backend ledger maps 15 lowering families to 22 passing integration tests
per backend. B-007-final-branch-audit retains current source hashes and execution
logs. Alias tests verify existing occurrence normalization; failed authoring logs
retain incorrect expectations and do not establish a compiler defect. Duplicate
and NUL structured member names and unknown selected page-key meaning refuse.

Truss NativeReview now accurately declares scalar/application and admitted storage
home scope while remaining Candidate. Its focused registration test preserves SQL,
parameters, columns, plans and pins for scalar and application fixtures. Native
supported-profile evidence joins, final composition/embedding and acceptance remain
open; these checks do not promote support.

## Current registration qualification checkpoint

B-007-final-branch-audit records graph (18 tests) and backend lowering
(22 integration tests per backend) semantic-family accounting. These finite
ledgers replace the remaining generic graph/lowering review task.
B-007-native-registration-join now proves exact registration correspondence on
1,025 native-tested artifacts (1,024 compiled, one identical refusal), preserving
query/integrity SQL and all meaning-bearing outputs. Ten semantic controls refuse.
The registration manifest comparison exposes a concrete uncovered capability:
`and` has no required-operation native case in either joined corpus. Next execute
conjunction truth tables, then finish selected domain/home qualification, supported
manifest/assessment declarations, final Python/browser composition and the
30-criterion acceptance audit. Candidate correspondence is not support promotion.

## Conjunction native gap closed

B-007-conjunction-native passes 32 truth-table/bag cases, both dialects and
admitted props/typed homes, with 16 PostgreSQL rollback transactions and 51
terminal Databricks statements. Independent reconciliation and eight semantic
controls pass. Every declared capability ID is now native-observed for each
review registration when joined with the prior corpus. Domain qualification
remains separate: broad Truss numeric evidence used its original-definition
backend, so verify those scalar domains through this exact registration before
promoting it. No owner-layout dependency or distribution license gate is added.
