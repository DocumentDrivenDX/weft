# B-005 native preparation — 2026-10-06

**Outcome:** engine boundary and emission primitive tests pass. B-005 remains
incomplete: no approved Truss binding or registered storage adapter exists.
This record does not close US-003 or the production qualification gate.

## Native environment

An isolated disposable Docker container `weft-b005-pg17` uses official image
`postgres:17.9`, digest
`sha256:2a0d0fe14825b0939f78a8cad5cd4e6aa68bf94d0e5dd96e24b6d23af4315545`.
PostgreSQL reports `17.9 (Debian 17.9-1.pgdg13+1)` and UTF8 encoding. No host
ports or host data volumes are published; the data directory is tmpfs. Startup
uses locale C. Other task containers/databases were not modified.

## Executed observations

- `cargo test -p weft-postgresql --locked --offline`: three tests pass.
  Identifier components are quoted individually, embedded quotes doubled,
  UTF8 byte limits checked without clipping and NUL/empty names refused.
  Values remain exact text in contiguous slots. Catalog int/smallint domains
  retain negative/zero IDs, refuse overflow and noncanonical/injected text,
  and remain separate from logical keys/allocation policy.
- The independent engine probe and Python Decimal checker pass nine boundary
  observations: exact numeric SUM, UTF8 C ordering, trailing-space distinction,
  numeric versus lexical cursor ordering, JSON absent/null/array states and
  empty count/SUM. No stored Truss data or generated adapter SQL was used.
- The Rust parameter example emits four slots. Native SQL PREPARE/EXECUTE returns
  exact `-2147483648`, `-32768`, `18446744073709551615` and the authored
  injection-like Unicode text under its correctly quoted alias. The harness
  escapes trusted fixture values to invoke SQL EXECUTE through psql; this is
  **not** evidence of host driver protocol parameter binding.

## Reproduction

Build the example with `cargo build -p weft-postgresql --example parameter_probe
--locked`. Run `python tests/truss-postgresql/prepared-check.py` against the named
isolated container. Feed `native-boundaries.sql` through `psql -U postgres -X -A -t
-v ON_ERROR_STOP=1` and pipe the one JSON row into `check-native-boundaries.py`.
Generated observations are retained in `target/b005/native-boundaries.json` and
`target/b005/prepared-summary.json`; no credentials are included.

## Remaining gate

Truss owner worktree now proposes ADR-004 document-qualified ownership but has
not accepted that storage change or supplied a versioned Weft exporter/binding.
TD-003 records alternatives and required physical/semantic correspondence.
The pending owner decision is not resolved by a working PostgreSQL instance.
Full original/application corpus, whole-value codecs, relationships, read-context/
role enforcement, protocol parameters and linked Python/browser adapter parity
remain unimplemented/unexecuted. B-006 cannot begin before B-005's PR merges.

## Source hashes

| Source | SHA-256 |
| --- | --- |
| `crates/weft-postgresql/src/lib.rs` | `5cd0752e29f2236fcac19e47e314fa28a087962808e2ec70d469208ea2260861` |
| `crates/weft-postgresql/examples/parameter_probe.rs` | `f6d0cf77b6315f7e12ba1ce281c1cf9ce294bbe03743e521c0e5a7e5685488e3` |
| `tests/truss-postgresql/native-boundaries.sql` | `3aea470e84ced366b0a2c4979ebba7d7973fe1402b1c5a81c35db07e4aa0fc63` |
| `tests/truss-postgresql/check-native-boundaries.py` | `27804d648936c9e17a11224dc34d7eeaccba62e568dbb65d617eca4f6a5bec45` |
| `tests/truss-postgresql/prepared-check.py` | `ab061813e529badcf7acb5a100f21679c8c6be8d7805d5a1eb01a164b94471dd` |

## Resumed candidate implementation — 2026-10-06

Owner direction authorizes development against the versioned candidate while
production adoption remains separate. Frozen upstream schema closure/source pins
live under `tests/truss-postgresql/upstream/`; no compiler fetch is performed.
Candidate artifact integrity checks and property-home admission are implemented:
exact original bytes/hashes, profile string, shape, duplicate mapping/native domains,
props/row correspondence and refusal of silent home fallback. Six Rust tests pass
(three primitive and three candidate-binding tests).

Generated scalar access now has JSONB and native row-store templates. The
independent `native-homes.py` witness executes both on PostgreSQL 17.9 with exact
values `0`, `2`, `2`, `10`, `18446744073709551615`; duplicates and numeric ordering
agree. A removed row payload yields one integrity violation rather than a dropped
owner or absent property. Temporary witness tables are local to each connection.
They are not an installed/adopted complete Truss layout.

These templates return integrity prerequisites separately from user filters.
Exact stored-domain admission must precede unsafe casts in the same qualified
view; templates do not themselves enforce that host protocol. Binding profile-pin
registration, model/catalog/source correspondence, codec/full domain evidence,
optional/null/recursive values, joins/aggregates/relationships, core Registry
integration and cross-runtime compiler qualification remain ongoing B-005 work.
No registered production compiler support follows from these partial witnesses.

## Registered candidate relational lowering — 2026-10-06

`candidate::Candidate` now implements the existing Rust Backend trait, with an
explicit `truss.postgresql` / `0.1.0-candidate` / `pg17.9-candidate` profile.
Its current manifest advertises only the 0.1 scalar relational IR, always candidate:
scan, projection, equality/AND filters, inner joins, grouping and SUM. It requires
explicit opt-in and retains host context/domain/integrity obligations. It changes
neither the frontend nor generic parameter/result ABI. Application IR 0.2 is not
advertised until its lowering/qualification is implemented; this is ongoing work,
not a reduced B-005 endpoint.

The compiler compares the decoded original binding model bundle to all supplied
ModuleInputs, checks selected mappings, and lowers owner/field identities through
props or typed row access. Scan type discriminators and all literal/member values
use ordered typed parameters; labels and schema/occurrence identifiers are quoted.
Field access is reused within a query. Result numbers are exact text. Row joins
retain optional-side ownership; integrity SQL is attached as a prerequisite rather
than a user filter. Complete scalar facet/domain verification remains a separate
host obligation, not inferred from root/payload shape checks.

The common emission checker exposed incorrect primitive integer facet metadata;
that metadata now uses the existing `integerWidth` facet structure. The complete
registered artifact passes common type/parameter/column validation.

Eight Rust tests pass (three primitive, three binding and two compiler cases).
The independent original 13 relational queries/results are retained and run across
both physical homes, producing 26 actual registered compiler/native executions.
All pass on PostgreSQL 17.9, including the sample Customer/Orders join/SUM,
empty global/grouped aggregates, Unicode spelling, trailing spaces, empty text and
injection-like stored strings. Fixture tables include an unrelated object type
with malformed lookalike property content; it does not enter results. Exact
numeric comparisons use Python Decimal and bag Counter, without float conversion.

The native fixture harness uses SQL PREPARE/EXECUTE through psql, explicit SQL-null
transport and one repeatable-read transaction for integrity/data observations.
This is not host driver protocol qualification, full Truss layout installation,
production profile adoption or complete corruption/policy enforcement evidence.
The integrity and data commands are executed in the fixture sequence; a production
executor must independently fulfill the reported obligations before publication.

Reproduce with `generate-compiler-fixtures.py`,
`cargo build -p weft-postgresql --example compile_probe --locked`, and
`python tests/truss-postgresql/compiler-native.py` against the isolated container.
`target/b005/compiler-reports.json` retains compiled artifacts and native rows.
Application whole-value/page/count/relationship lowering, full corpus expansion,
model-to-physical inventory/codec correspondence, host enforcement and candidate
Python/browser parity remain required before closing B-005.

### Application relational component evidence (2026-10-06)

The candidate now lowers application IR for scalar projections, optional scalar
presence, COUNT, SUM, equality joins and filters, typed ordering, named parameters
and LIMIT. The manifest declares those candidate capabilities for dialect 0.2;
recursive values and relationship reads remain refused pending implementation.
This does not complete B-005 or qualify a production binding.

`generate-application-fixtures.py` preserves the original application model and
six authored application queries, then adds an optional-nickname projection.
Each runs against both homes, giving 14 component cases. Nine Rust tests pass,
including all 14 public compile requests and typed result-contract assertions.
The previous 26 relational compile cases remain passing.

`application-native.py` executes all 14 emitted queries on the isolated PostgreSQL
17.9 container with independently authored expected rows and the existing
application data. Counts, grouped counts, equality joins, exact decimal SUM,
named parameter transport, and optional absence/present-empty strings pass for
both homes. Missing row state preserves its owner row and produces the explicit
absent envelope; a stored empty string produces a value envelope. This evidence
does not treat optional availability as permission for native null.

Native artifacts and rows are retained in
`target/b005/application-native-reports.json`. Reproduce with the application
fixture generator, the compile_probe build, and application-native.py. As with
the previous native harness, SQL PREPARE/EXECUTE does not qualify a production
driver protocol, policy enforcement, or physical-inventory/codec correspondence.
Authored page-key binding correspondence and snapshot continuity remain open;
the emitted candidate host obligations do not prove either requirement.

### Authored key and source correspondence (2026-10-06)

Selected entity/property source documents and accepted definitions now compare
against the original pinned ModuleInputs, preserving unknown extension content
in the comparison. A self-consistent replacement digest does not authorize a
different selected definition. Page-key admission requires the source owner's
mapped type, authored key ID, ordered mapped property IDs, and the accepted key
definition from the source entity. Missing keys, wrong owners/components,
reversed composite components and rehashed substituted definitions refuse with
WFT-BINDING and no SQL or query parameters.

The application fixture generator now emits authored key mappings and includes
the original composite-cursor query. Twelve Rust tests pass, retaining the 26
original relational compile cases and expanding application compile/native
coverage to 16 cases across both homes. All 16 execute successfully on PostgreSQL
17.9 with independently authored result rows; composite comparison uses typed
numeric and C-collated string operands, with their authored tuple order.

These checks prove selected source/key correspondence only. The candidate
fixture comparison/encoding definitions are still unqualified placeholders;
physical inventory, codec/profile authority, actual stored-key uniqueness,
runtime integrity enforcement and snapshot continuity remain separate open
requirements. No production qualification or B-005 completion is claimed.

### Relationship existence component evidence (2026-10-06)

Forward and inverse HAS_RELATED now lower to correlated EXISTS over the candidate
edge layout, matching physical endpoint IDs/types and comparing each typed target
key component. Endpoint keys are validated separately against their own authored
types, key IDs and property order; no source/target arity equivalence is assumed.
Relationship source documents and accepted definitions must match original UMF.
Missing relationships, swapped/wrong endpoint roles and changed key/definition
artifacts refuse atomically. Thirteen Rust tests pass, including 36 relationship
mapping refusal variants across both directions and property homes.

Twenty application compile/native cases now pass on PostgreSQL 17.9. Forward and
inverse existence use the original four-edge fixture, preserving its duplicate
edge rather than silently deduplicating stored data. EXISTS returns each owner
once even when more than one qualifying edge exists. This is bag-semantics
component evidence; the minimal fixture does not claim installation of Truss's
production unique-edge constraints.

The compiler also emits a relationship-integrity host obligation with checks
for missing endpoints and mismatched endpoint types. The fixture executes these
checks before its data query in the same repeatable-read transaction. Four
additional native corruption observations (forward/inverse, props/row) verify
that a deliberately wrong source type produces one violation. The compiler
does not execute or enforce these obligations itself. Complete authorized
endpoint visibility remains required; policy-hidden endpoints cannot silently
be interpreted as relationship absence. Bounded related-key projections,
compound values and runtime/profile qualification remain unfinished.

### Bounded related-key projection evidence (2026-10-06)

RELATED_KEYS now lowers for forward and inverse relationships over both property
homes. The candidate selects typed target-key components in authored order,
sorts by native numeric/boolean or C-collated string expressions, and probes
bound+1 rows. It emits an items array and an explicit truncated boolean, retains
duplicate edges, preserves numeric key components as exact text, and supplies
the original typed key descriptor in result metadata. Empty relationships emit
an empty items array with truncated=false. Endpoint/source correspondence and
relationship integrity obligations also apply to these projections.

Thirteen Rust tests pass with 32 application compile cases and the original 26
relational compile cases. All 32 application cases execute on PostgreSQL 17.9
across props and row homes. New native cases cover forward/inverse bounded
results, duplicate-edge truncation, empty relationships, numeric ordering of 2,
10 and uint64 maximum, and user aliases resembling generated relation, edge or
row-store aliases. Generated edge/row aliases now avoid active scan names.
Each relationship-query fixture additionally observes one integrity violation
for a deliberately corrupt endpoint before any data-query publication.

The minimally seeded edge bag remains component evidence rather than proof of
production unique-edge layout conformance. Full recursive property values,
accepted inventory/codec authority, host enforcement, candidate Python/browser
parity and the remaining B-005 test gates still need implementation/evidence.

### Scalar-item sequence component evidence (2026-10-06)

Required sequences with required scalar items now lower from JSONB arrays or
complete-value-tree row bindings. JSONB iteration preserves item ordinal; row
iteration orders typed scalar payloads by sequence_ordinal and checks zero-based
contiguous ordinals. Empty arrays remain present values. Exact numeric leaves
become JSON strings without a float intermediate. Sequence columns retain the
original identity-based value descriptor. Scalar-root row access refuses rather
than providing a lossy fallback.

Fourteen Rust tests pass, with 36 application compile/native cases and the
original 26 relational compile cases. PostgreSQL 17.9 executions cover duplicate
string items, empty arrays, empty string items, integer items above float
precision, and uint64 maximum across both homes. Four additional corruption
observations count one violation for a wrong JSON item type or a gapped row
ordinal before data publication. Container roots cannot carry a scalar payload.
The minimal row fixture now declares sequence slots/ordinals; it is still not an
installed or qualified production Truss layout.

This is the first compound-value implementation stage. Optional sequence
envelopes, recursive item descriptors, maps and structured values remain
unfinished. Exact stored facet/domain qualification, complete tree integrity,
inventory/codec authority and runtime host enforcement remain open; candidate
component evidence does not close B-005.

### Optional collections and scalar-item maps (2026-10-06)

Optional sequences now emit explicit absent/value envelopes and preserve a
present empty array. Optional JSONB presence is undefined for an invalid root
container rather than falsely reporting absence; its integrity observation then
refuses publication. Explicit native null still requires separate qualification
and is rejected by these candidate sequence/map observations.

The sequence access implementation is consolidated into collection.rs and now
also handles scalar-item maps with required or optional roots. Candidate fixture
map keys are exact strings; C comparison verifies row-key uniqueness without
normalizing case, whitespace or Unicode spelling. JSONB map iteration and native
row map slots retain exact numeric leaves as JSON strings. Empty maps remain
present values; absent maps retain their owner row and an absent envelope.
Duplicate row keys are an integrity violation before aggregation/publication.

Fourteen Rust tests pass with 44 application compile/native cases plus the
original 26 relational compile cases. PostgreSQL 17.9 results verify optional
sequence absence/present-empty, required/optional string maps, numeric maps above
float precision and at uint64 maximum, empty keys/values, case differences,
trailing spaces and distinct composed/decomposed Unicode keys. Additional native
observations detect explicit-null optional sequence/map roots, duplicate map
slots and wrong JSON map item families. The minimally seeded node fixture now
declares map_key with C collation; this is not production layout installation.

Recursive item/structured values, complete tree integrity and accepted native
codec/inventory authority remain open. String-key map semantics are the explicit
candidate fixture codec, not an inference from UMF cardinality alone or a claim
of production profile adoption. Runtime host enforcement and Python/browser
candidate parity remain required before closing B-005.

### Structured scalar members and whole-entity evidence (2026-10-06)

Structured scalar-member properties now lower from candidate JSONB objects or
complete row-tree roots. Members follow the original identity/type descriptors;
optional members get their own presence envelopes. Numeric leaves retain exact
text. Candidate row members match exact UTF-8 canonical logical-identity bytes;
the native fixture declares record_field_identity_bytes separately from authored
member names. This fixture encoding still requires eventual owner codec authority.
Unknown member observations, missing required members, duplicate row members,
wrong scalar payload families and unqualified native null are integrity failures.

Whole-entity projection is now declared as a candidate capability. The authored
whole-entity, scalar-page, large single-cursor and related-filter cases compile
and execute across both homes. The original large cursor correctly produces no
rows against the independent three-customer fixture; an added cursor=1 case
proves the positive two-row whole-entity page rather than narrowing the authored
query or changing its parameter. The original fixture's address.street and
optional address.zip keep their declared meaning, including nested absence.

Fifteen Rust tests pass with 58 application compile/native cases and the original
26 relational compile cases. PostgreSQL 17.9 additionally verifies a nested
optional uint64 maximum as exact text. Six structured corruption observations
(three per home) detect missing required street, unknown member identity/name,
and explicit null in optional zip before any data result publication. Numeric
structured descriptors retain their original identity, absent-allowed state and
64-bit unsigned facet in the public result contract.

This stage covers scalar members of structured records. Recursive item/member
shapes and cyclic descriptor traversal remain unfinished. Complete tree integrity,
accepted physical inventory/codec authority, host enforcement and candidate
Python/browser parity still gate B-005. Truss's reported packed-toolkit host
boundary/optional compiler lifetime does not itself adopt this candidate binding
or require an ABI change; direct Truss operations remain independently owned.

### Complete-tree structural observations (2026-10-06)

Complete-value-tree collection/structured access now adds a structural
observation scoped to its selected state and root. A recursive UNION visits
each reachable node once. All nodes must be reachable; every payload must belong
to a node; scalar nodes require exactly one payload and containers/null nodes
require none. Root/child slot roles must agree with their parent kind, scalar/null
nodes cannot parent children, and competing scalar payload columns are invalid.
An owner/property cannot have multiple selected states. This does not yet add
the same complete-tree observation to the distinct scalar-root access profile.

Fifteen Rust tests and all 58 native application cases remain passing on
PostgreSQL 17.9. Six additional sequence row-tree topology observations detect
orphan nodes, disconnected cycles, orphan payloads, a child under a scalar,
competing typed payload columns and duplicate owner/property states. The first
five report one violating owner observation; two competing states report two.
The corruption harness sets a two-second statement timeout, and the disconnected
cycle completes with a violation. Exact observations are retained in the
sequence-page-row topology receipt in application-native-reports.json.

These checks improve complete-tree structural evidence; they do not qualify
the remaining recursive descriptor codec, full exact native facets, source-byte
authority, policy visibility, installed constraints or host enforcement. B-005
remains open for recursive item/member/cyclic descriptors and its other gates.

### Recursive JSONB descriptor-graph codec (2026-10-06)

JSONB compound projections now use a finite descriptor graph rather than expanding
the schema recursively at compile time. Native SQL walks the actual finite stored
value, following item/member identity references and applying exact numeric text
and nested presence encoding from the descriptors. Structured references reuse
their record members, including cyclic type references. Unknown members, wrong
families and unqualified native null remain integrity violations. Object codecs
refuse duplicate authored member names and names unrepresentable in PostgreSQL
JSONB instead of collapsing distinct member identities.

The observation requires depth below 128 and at most 100,000 observed nodes. These
are explicit candidate runtime domain limits; reaching a bound is an integrity
failure before publication, not permission to publish a truncated value. Full
numeric facet/source codec authority and host enforcement remain separate gates.

Sixteen Rust tests pass, including the duplicate-member identity refusal. Native
PostgreSQL 17.9 retains all 58 paired application cases and adds two JSONB-only
recursive probes: nested sequences with integers above float precision/uint64
maximum, and a cyclic UMF structured descriptor applied to a finite two-level
value. Nested optional absence and exact numbers survive. These 60 executions
comprise 31 props and 29 row cases, not recursive row-tree qualification. The
recursive props probes do not seed an asserted alternative row codec. Existing
corruption and topology observations remain passing.

Equivalent descriptor-graph row-tree decoding is required next. Static row
lowering remains available for the already evidenced scalar-item collections and
scalar-member structured shapes. This staged JSONB implementation does not
replace the requested native row/column scope or close B-005. Candidate host
runtime parity, complete accepted binding/profile authority and the remaining
test gates are still open.

### Recursive native row descriptor codec (2026-10-06)

Complete-value-tree row projections now traverse the same finite UMF descriptor
identities as props projections. Native nodes remain the source: scalar payloads
come from typed row columns, sequence slots from ordinals, map slots from keys,
and record slots from exact member identity bytes. A temporary JSONB result
carrier connects the native traversal to the common exact-value encoder; it does
not read a property's props home or permit a storage fallback.

The native observation rejects unmapped nodes, duplicate paths, mismatched native
kinds, non-dense sequence ordinals and competing slot metadata. Existing tree
checks also reject disconnected cycles, orphan payloads, scalar children and
competing state roots. Traversal keeps explicit depth and node-count bounds.
The fixture writer independently follows original UMF modules to populate native
rows; its member identity encoding is candidate fixture evidence, not an adopted
Truss producer codec.

Fresh verification passes all 16 Rust tests and 62 PostgreSQL 17.9 application
executions: 31 props and 31 native row cases. Both homes now include nested
sequences and finite values whose UMF structured descriptor references itself.
Exact numeric text and nested optional presence agree with independent expected
values. Existing corruption observations continue to pass.

B-005 remains open. Stored numeric facet checks, candidate Python/browser
runtime integration, full application corpus coverage and accepted binding,
codec and host-enforcement qualification remain required. This evidence permits
continued candidate work and does not claim installed Truss compatibility.

### Compound numeric facets and runtime composition (2026-10-06)

The common descriptor codec checks integer integrality and signed/unsigned width
bounds, plus decimal scale and precision magnitude, before publication. Casts
sit behind JSON numeric-family CASE branches. Native row payloads reach these
checks through the temporary result carrier, retaining their storage home.
Six added corruption executions reject -1, uint64 maximum plus one and 1.5 in
an unsigned integer sequence across both homes. All 62 valid native executions
and all 16 Rust tests remain passing. Decimal corruption boundaries and scalar
projection/filter facet coverage still require further execution evidence.

An optional `truss-postgresql-candidate` build feature now registers the adapter
in the shared runtime and is forwarded by Python and WASM crates. Defaults still
register no candidate adapter. A Rust runtime check with both candidate and
independent test-backend features passes. This is composition evidence only;
fresh native Python and browser WASM execution remains required.

### Candidate Python and browser parity (2026-10-06)

The candidate-feature native wheel built with Rust 1.90, Maturin 1.9.6 and PyO3
0.27.1 executes on Python 3.12.14. All 62 application requests match the actual
native compiler responses byte-for-byte, with PATH empty and subprocess entry
points disabled during compilation. Invalid transport inputs retain the public
boundary refusals. The abi3 wheel targets Python >=3.9; only 3.12.14 execution
is evidenced here.

The candidate-feature wasm32 build executes in Chromium 153.0.8010.12 through
Playwright 1.62.1 and the existing browser wrapper. The same 62 requests match
native response bytes. WASM imports contain only generated error/externref
interop, no WASI. No Node globals exist in the page. Network APIs are disabled
after loading the compiler, and wrapper trap retirement remains passing. Debug
WASM is 10,333,098 bytes (SHA-256
`1df331583c92ef8e25202ea2d5c5e724830129b8414ceed264d21c3d81cf3052`);
observed memory grows from 1,835,008 to 3,276,800 bytes. These are observations,
not release size or performance budgets.

Reproduction uses `scripts/run-b005-embeddings.sh` following a fresh
`tests/truss-postgresql/application-native.py` run. Machine-specific toolchain,
Python and Chromium locations belong in host environment variables. Initial
sandboxed Chromium launch failed at macOS Mach IPC registration; the authorized
unsandboxed launch completed. No database connection or execution is owned by
the compiler bindings. Full corpus parity, complete source/binding authority,
scalar facet coverage and host enforcement remain open before closing B-005.

### Scalar numeric domain observations (2026-10-06)

Scalar access now emits exact integer integrality/width and decimal scale/magnitude
predicates in both storage homes. Props casts are guarded by numeric-family CASE;
row predicates operate directly on native numeric columns. These remain required
same-view host integrity observations before user predicates, aggregation or
publication; they are not filters that silently discard corrupt objects.

All 16 Rust tests and 62 native application executions pass. Six additional
corruption executions, applied to a selected decimal SUM input, detect scale
excess (1.001 for scale 2) and either signed precision-magnitude boundary for
precision 28/scale 2 in both homes. A duplicate harness block initially used the
previous request's observations; removing that block and rerunning yields the
recorded passing results. Candidate embedding parity must be refreshed after
this SQL change; preceding byte-parity evidence applies to the preceding build.
Broader scalar boundary and complete corpus/host qualification remain open.

### Scalar complete-tree access and refreshed embeddings (2026-10-06)

Scalar fields now accept a `complete-value-tree` row binding in addition to the
separately assessed scalar-root binding. Lowering retains native typed scalar
expressions for predicates and SUM and adds complete-tree integrity observations.
It does not substitute encoded JSON numeric strings for numeric SQL operands.
Native fixture scalar roots now carry explicit root slot metadata. Two additional
cases exercise a decimal SUM and whole-entity projection with all scalar row homes
using complete-tree access, preserving the original independent expected results.

All 16 Rust tests and 64 actual PostgreSQL application cases pass (31 props,
33 row). The decimal corruption observations also pass under complete-tree SUM.
Fresh Python 3.12.14 and Chromium 153.0.8010.12 checks each match all 64 actual
compiler response strings byte-for-byte. The existing disabled subprocess/network,
invalid transport and trap retirement checks remain passing. Current debug WASM
is 10,349,915 bytes, SHA-256
`17ab79091b1f8fb7c78136c2b5693a62f02292cc7e5f7fe5a4a20fc0490a9c54`.

This closes the scalar complete-tree lowering gap, not the full story. Complete
application corpus coverage, remaining native boundary cases and accepted
binding/codec/source/host context qualification remain open. B-005 is not merged.

### Full application compiler corpus and host parity (2026-10-06)

`tests/truss-postgresql/full-corpus.py` applies a candidate binding to every one
of the 620 authored application inputs in each property home. All 304 resolved
inputs compile in both homes; all 316 authored frontend refusals preserve their
expected diagnostic in both homes. None of the resolved inputs remain backend
refusals. Invalid model fixtures retain original input bytes rather than being
repaired to obtain a successful decision. Candidate mappings remain synthetic.

All 1,240 decision strings also match byte-for-byte in the native Python 3.12.14
wheel and real Chromium 153.0.8010.12 WASM. Disabled subprocess/network, invalid
transport and browser trap retirement checks remain passing. Reproduction is
`scripts/run-b005-full-corpus.sh` after fresh candidate builds and native reports.
The harness reports backend refusals of resolved inputs separately instead of
counting them as supported compilation.

This is full compiler/embedding decision coverage, not full native database
result coverage. The native execution corpus remains 64 valid application cases
plus its corruption observations; parameter-boundary execution and additional
independent Unicode/large-key result scenarios still require native evidence.
Binding authority and host enforcement gates also remain open. B-005 stays open.

### Native parameter corpus and key ordering expansion (2026-10-06)

`tests/truss-postgresql/parameter-native.py` executes all 288 resolved authored
parameter-boundary inputs in each home: 576 native equality queries. Independent
fixtures store each exact boundary number and use a deliberately different
storage object ID. All typed integrity observations report zero violations and
each query returns its authored integer/decimal value exactly, checked with
Python Decimal. This covers signed/unsigned widths and decimal precision/scale
cases without binary float. Refused parameter inputs remain covered by the
1,240 compiler/embedding decision corpus and are never executed as data queries.
The transport uses psql PREPARE/EXECUTE with trusted fixture literals; it does
not qualify a host driver's wire parameter protocol.

The main application native corpus now passes 70 cases (34 props, 36 row). Six
added executions retain composed/decomposed Unicode distinctions, empty text,
trailing spaces, C-collated composite cursor order and exact numeric key order
above float precision through uint64 maximum. Expected ordered rows come from
the independently authored application oracle. Combined native result coverage
is 646 valid application executions, plus corruption observations. Updated
70-case embedding summaries still need refreshing; the full authored compiler
corpus's prior 1,240 parity decisions remain scoped to that unchanged corpus.
Binding/codec authority, host enforcement and remaining boundary gates stay open.

### Empty aggregate and ordered-row gates (2026-10-06)

Six added native executions cover empty global COUNT, grouped COUNT and SUM in
both homes. Expected results are independently fixed as one zero row, no rows,
and one nullable result respectively. No aggregate zero/null substitution occurs.
The application harness previously compared all results as bags; it now also
compares ordered query rows as sequences. A fresh run with that stronger assertion
passes all 76 application cases (37 props, 39 row), including Unicode/composite
cursor and large-key ordering. Bag equality alone is not retained as order proof.

All 16 Rust tests pass. Fresh Python 3.12.14 and real Chromium 153.0.8010.12 each
match all 76 actual compiler responses byte-for-byte, retaining disabled host IO
and transport/trap checks. Combined with the separately evidenced 576 native
parameter-boundary executions, native valid result coverage is 652 executions,
plus corruption observations. The full authored compiler corpus remains 1,240
decisions; those counts represent distinct verification scopes, not additive
story acceptance. Complete binding/codec authority and host context enforcement,
remaining native boundaries and story qualification remain open before B-005 PR.

### Scalar payload exclusivity and owner boundary review (2026-10-06)

Scalar-root row access now requires all competing typed payload columns to be
NULL. A new native decimal SUM corruption observation injects text alongside a
numeric payload and detects one violation before publication. All 16 Rust tests
and 76 native application cases pass. Embedding parity must be refreshed for
this new SQL build; preceding parity receipts remain version-scoped.

Reviewed the current uncommitted Truss CONTRACT-007 draft (SHA-256 `eb0527ae0f2dc8b1b4af8194deb3c377947036427622096d0ac22dadcd29a97a`). It still separates original artifact/profile custody from live role,
affine transaction, snapshot and publication admission. Its optional-state rules
require complete authorized visibility; hidden state cannot establish absence.
No production binding adoption or new compiler ABI is inferred from this review.
Frozen Weft schema snapshots are unchanged. Remaining gaps include semantic
physical inventory/codec/profile qualification, duplicate scalar-root observation
and native host-context enforcement evidence. Candidate continuation remains
authorized; reviewed draft prose is not owner acceptance.

### Scalar-root uniqueness observations (2026-10-06)

Scalar row lowering now observes exactly one selected property state, one root
node and one matching typed payload. Correlated counts keep ambiguity visible;
the data query does not deduplicate owner occurrences or pick an arbitrary row.
Three native corruption probes remove the fixture uniqueness constraint and
insert a duplicate selected state, root or payload. Each generated observation
reports two violating joined occurrences before any data result is published.
This evidence is about observation behavior, not installed owner constraints.

All 16 Rust tests and 76 native application cases pass. Fresh Python 3.12.14 and
real Chromium 153.0.8010.12 each retain byte parity for the same 76 requests after
the exclusivity and uniqueness SQL changes. Full 1,240-decision parity must also
be refreshed before final story qualification. Hidden optional state still needs
complete authorized visibility supplied by the live host; native row counts do
not create that authority. Physical inventory/codec/profile qualification and
host-context enforcement remain open. B-005 has not yet been committed or merged.

### Physical home/template correspondence (2026-10-06)

The fixed candidate emitter now refuses home selectors that name a different
record kind, props relation/column/discriminator/accessor or native state/node/
scalar target. Previously those decoded home fields could differ while lowering
still used the fixed template names. Checks are confined to the versioned
candidate profile; they do not define a universal physical identity vocabulary
or qualify the complete layout inventory. Accepted owner profiles require their
own original inventory/codec/association semantic admission.

Seventeen Rust tests pass. A new mutation test rewrites and rehashes each of
12 physical home fields independently; exact artifact hash validity cannot make
the redirected home usable. Existing selected logical/model/key/relationship
custody tests and all 76 application compilation cases remain passing. SQL for
unchanged admitted homes is unchanged, so no broader native execution or new
embedding qualification is inferred from this run. Before closing B-005, full
host parity refresh and physical codec/profile/authority gates remain open.

### Explicit candidate visibility/context obligations (2026-10-06)

The existing host-owned `truss.candidate.context` obligation now declares complete
state visibility before absence decoding and complete child visibility before
compound decoding. Hidden rows are never logical absence. Its execution fields
require per-execution pin checks, affine same-transaction integrity/data access,
current authority/disclosure before publication and refusal of unknown obligation
meanings before SQL. Separate pages claim no snapshot continuity. This clarifies
the versioned candidate bridge meaning; it does not introduce live authorization
or move database execution into the compiler.

Eighteen Rust tests pass. The public compile result for optional property pages
in both homes retains this obligation and its host owner. This test proves
artifact carriage only. Native RLS/authority/transaction/publication enforcement
must still be demonstrated by a qualified host procedure; neither these fields
nor the existing synthetic integrity harness prove it. Changed obligation bytes
require fresh full native-report/Python/browser parity before B-005 completion.
Physical codec/profile/source authority remains separately unqualified.

### Consolidated current-state B-005 audit (2026-10-06)

A fresh current build passes 26 original relational native cases, 76 application
native cases and 576 parameter-boundary native executions. All 1,240 authored
application compiler decisions pass with zero resolved-input backend refusals.
Native Python 3.12.14 and Chromium 153.0.8010.12 refresh byte parity on both the
76-case native application corpus and the complete 1,240-decision corpus after
physical-home admission and visibility/context obligation changes. Existing IO,
transport and trap checks remain passing. Eighteen Rust tests have current evidence.

| Story gate | Current evidence | Remaining proof |
| --- | --- | --- |
| US-003-AC1 native corpus | Independent exact rows, types, multiplicity and order across both candidate homes | Adopted owner storage/engine binding and original installed-profile correspondence |
| US-003-AC2 mapping boundaries | Exact logical/model/key/relationship custody, unrelated type exclusion, rehashed physical-home refusals | Full inventory/join/association semantic admission against accepted original owner resources |
| US-003-AC3 semantic refusals | Full authored refusals; native null, unknown selected homes and ambiguous physical targets do not silently coerce | Accepted codec/value/presence/stored-domain profiles and remaining authority-qualified unsupported subsets |
| US-003-AC4 host execution | Exact text/Decimal carriers, generated integrity observations and explicit context/visibility obligations | Actual driver preparation and native live policy/revision/affinity/publication enforcement or pre-SQL refusal |

These gates remain partial. Synthetic fixtures, hash custody and psql trusted
PREPARE literals cannot supply owner adoption, installation or driver authority.
An adopted Truss profile reference was requested while candidate work continues.
No story acceptance, B-005 completion or production compatibility is claimed.
B-006 must wait until the complete B-005 slice is committed, reviewed and merged.

### Actual PostgreSQL driver preparation (2026-10-06)

`tests/truss-postgresql/driver-native.py` now executes all 576 valid authored
parameter-boundary requests through psycopg 3.2.10 with libpq 17.5 (170005).
A separate owned PostgreSQL 17.9 container used the same pinned official image,
UTF8/C initialization and a random localhost-only port; it was removed after the
successful test. The existing native-fixture container was untouched.

RawCursor submits generated `$n` SQL unchanged with `prepare=True`. Host parameters
use Decimal for exact numeric slots, and selected model values deliberately differ
from storage object IDs. Integrity observations and data queries use one host
connection/transaction. All 576 results arrive as PostgreSQL text (OID 25), remain
Python strings until explicit Decimal comparison, and equal their independently
specified values exactly. No SQL literal substitution, JSON-number parsing or
binary float participates in generated-query execution. Reproduction requires
an owned local PostgreSQL 17.9 instance and `WEFT_DRIVER_PORT`, with pinned
`psycopg[binary]==3.2.10` in the test venv. Core has no driver dependency.

This qualifies the tested driver preparation and numeric transport subset. It
does not qualify live policy/disclosure, catalog revision, held snapshots or
publication admission. Temporary fixture tables are synthetic; no installed
Truss binding adoption is inferred. The US-003-AC4 driver gap has evidence, while
its policy/revision enforcement gate and owner profile qualification remain open.

### Registered selected row obligations (2026-10-06)

Candidate assessment now resolves the selected row home's storedDomainObligation
rather than discarding it and always substituting the candidate context. Only
`truss.candidate.context` is registered for this versioned candidate. An unknown
selected obligation refuses before SQL. This is candidate bridge correspondence,
not adoption of any production stored-domain verifier or arbitrary owner ID.
Unselected model/extension content remains preserved without claiming support.

Nineteen Rust tests pass. A public compile mutation test changes the selected
SUM property's obligation, rehashes both home artifact and binding, and confirms
WFT-BINDING with no SQL. All 76 valid application requests still compile. SQL and
obligation bytes for unchanged registered fixtures are unchanged; this test adds
admission refusal evidence, not new native policy/revision qualification. Accepted
owner resource meanings and live enforcement remain open before closing B-005.

### Owner publication check and candidate checkpoint (2026-10-06)

GitHub's current Truss default branch is `d3dcddeec888cae4336b18af43a31fd19b93cf79`.
The published contract inventory contains CONTRACT-001 through CONTRACT-005 and
layout/module-isolation SQL; requesting CONTRACT-007 returns HTTP 404. The open
Truss PR list is empty. This check establishes that the required adopted execution
and binding artifacts were not found in those current remote surfaces; it does
not claim every private or unpublished owner artifact is absent.

The tested candidate branch is checkpointed for preservation. The checkpoint is
not B-005 completion, story acceptance, owner adoption or permission to begin
B-006. Qualification still requires the adopted original binding/codec/bridge
resources and actual live policy/revision/authority execution evidence. The
profile-reference question remains pending while independent candidate work
continues where it can advance those requirements.

### Selected codec/profile registration (2026-10-06)

Selected property value and presence pins must now match the explicitly registered
synthetic candidate profile and its empty fixture definitions. An unknown profile
cannot reuse the candidate type-directed codec merely because artifact hashes and
home/property correspondence are valid. This is a restriction of the candidate
profile; it does not adopt the synthetic placeholders as Truss owner resources.
A future accepted owner profile requires its own original definition interpreter
and native evidence, not renaming an arbitrary profile into this fixture pin.

Twenty Rust tests pass. Four public compile mutations change the selected value
or presence profile across both homes. Props mutations also update and rehash the
home's corresponding profile, preserving correspondence so rejection specifically
exercises registration. All refuse with WFT-BINDING and no SQL. All 76 unchanged
application fixtures still compile. SQL/result/obligation bytes for registered
fixtures are unchanged; no new owner/native authority is inferred. B-005 stays
in draft PR #7 with owner adoption and live host-enforcement gates open.

### Selected native join profile registration (2026-10-06)

A new public compile mutation test first demonstrated that an unknown native join
profile could still select fixed row joins. With a correctly rehashed home and
binding, the pre-fix response was compiled rather than the required refusal.
Candidate assessment now requires the registered synthetic join pin and exact
empty fixture definition bytes/hash. Unknown profile meanings refuse before SQL;
these restrictions do not qualify an original Truss producer or owner join body.

Twenty-one Rust tests pass after the fix, including all 76 unchanged application
requests. The focused test has recorded red/green evidence in host logs. An initial
command used an underscore instead of the declared hyphenated Cargo test target;
that invocation was corrected before obtaining the actual failing assertion.
Registered fixture SQL/result/obligation bytes are unchanged. Adopted owner join
and association resource interpretation and live host enforcement remain open.
The candidate remains draft PR #7, with no B-005 merge or B-006 start.

### Execution basis registration (2026-10-06)

A failing public compile test demonstrated that an unknown read-context profile
could compile. Candidate assessment now requires its registered synthetic layout,
identity, value, key, exporter and read-context pins plus the known empty fixture
read-context/inventory/layout/catalog definitions. Arbitrary execution requirements
cannot be discarded while candidate host obligations are substituted. These checks
make the synthetic profile boundary explicit, not production qualification.

Twenty-two Rust tests pass. Six independent execution-basis profile mutations
refuse with WFT-BINDING and no SQL after rehashing the binding. All 76 unchanged
application fixtures compile. The registered fixture SQL/result/obligation bytes
remain unchanged. Accepted owner definitions require a separately implemented
semantic interpreter and joint evidence; assigning the synthetic pin to genuine
owner data is not adoption. Live host policy/revision/publication enforcement and
owner source/resource qualification still gate B-005 completion and merge.

### Updated local-draft integration handoff (2026-10-06)

The current Truss worktree supplies concrete read-context, presence, recursive
value graph, JSONB leaf-codec and row-join grammars. Publication is not the
implementation dependency: these local drafts are available for pinned design
work. Adoption and native qualification remain separate final story gates.

Weft also has unfinished integration work. Its candidate currently registers
empty synthetic definitions; it must add original-definition interpretation and
complete graph/inventory/profile correspondence before these authored resources
can compile. This work cannot be replaced by assigning real owner definitions
the synthetic pin. Numeric-token readback and operation-specific comparator
assessment must be explicit; current JSON-number fixtures do not prove that
profile. Native row token/source columns need their original codec mapping,
independently of the already tested typed numeric projection. Read-context
requirements must resolve through actual registered bridge meanings, preserving
the existing parameter/decoder/obligation ABI.

Truss owns populated producer/registry/native enforcement resources and joint
profile adoption. Weft owns semantic admission, safe lowering and result-bridge
conformance against those originals. No new UMF capability or duplicate Truss
execution adapter is required. Full recursive/non-JSONB scope remains intact.
These next implementation inputs are draft resources pinned here for review:

- `truss-read-context-definition-v0.1.proposal.schema.json`: SHA-256 `01eef30067e6ce3ff3957475e163e8c9e870d1fa36b67c94aa6b2fe11d5e14cb`.
- `truss-presence-definition-v0.1.proposal.schema.json`: SHA-256 `08270a400dceb28e855ead142e31cfad1a30fd116d84b5380022fa3de4542f11`.
- `truss-value-definition-v0.1.proposal.schema.json`: SHA-256 `ae6d9227ddfe82bd531e3fb2754382f4bbe898ef20afc8690a197d78a16d62f3`.
- `truss-jsonb-leaf-codec-v0.1.proposal.schema.json`: SHA-256 `ba3088c60b154de39b1985da7d283ffdbb93cb4f59c0df4d80a12462f319de77`.
- `truss-row-join-definition-v0.1.proposal.schema.json`: SHA-256 `ddc7528dd546dfc76092076613fa612caee55a45ed51f46a553675b6f4e9d2b1`.

### Original presence-definition interpreter foundation (2026-10-06)

The current Truss presence grammar is frozen separately as
`upstream/presence-definition.schema.json`, SHA-256
`08270a400dceb28e855ead142e31cfad1a30fd116d84b5380022fa3de4542f11`.
A pure Rust definition interpreter preserves exact original JSON and accepted
source bytes, checks closed grammar constants, registered-profile correspondence,
canonical base64/digest custody and byte-for-byte authored-definition agreement.
It has a candidate four-MiB definition transport bound.

Independent presence tests first failed against the unimplemented admission.
The implementation then passes absence, empty string/list/record, false boolean,
SQL/null/nonobject storage-root refusal, changed source/profile and altered
coercion/default meanings. Observation borrows original values rather than cloning
or normalizing them. Authored nonnullable JSON null is an integrity refusal;
authored nullable JSON null still refuses capability because separate native-null
qualification is not established. This preserves availability versus nullability.

Twenty-four Rust tests pass. This is a standalone interpretation foundation,
not new compile support: current candidate registration still selects synthetic
fixtures. Value graph, native join and read-context interpretation, complete
source/profile/resource correspondence and compile/result-bridge integration
remain required. No Truss host authority or profile adoption is inferred.

### Original read-context interpreter foundation (2026-10-06)

The original read-context grammar and acceptance dependency are frozen in a
separate offline schema bundle, with exact hashes in read-context-source-pins.json.
The root grammar SHA-256 is
`01eef30067e6ce3ff3957475e163e8c9e870d1fa36b67c94aa6b2fe11d5e14cb`.
Compile-time grammar validation uses that closed bundle without fetching resources.

A pure Rust interpreter preserves original JSON/resource bytes, validates exact
registered profile/resource correspondence and canonical base64/digest custody,
and resolves required obligation IDs against an explicitly supplied registration
set. Missing meanings refuse. Static requirements retain original binding/layout,
current authority, live affine transaction and publication admission in both
consistency modes; snapshot custody is required only for held_snapshot. A mode
not allowed by the original definition refuses instead of creating a snapshot.

Tests first failed against the pending interpreter, then pass conditional snapshot
requirements, live-only refusal, unknown obligations, duplicate modes, implicit
recompile policy, serialized transaction handles and changed original resources.
Twenty-six Rust tests pass. These prove static interpretation/correspondence,
not original host-procedure evidence, live enforcement or profile adoption.
Current candidate compilation remains on synthetic definitions; wiring real
read-context/value/presence/native-join meanings through a registered profile and
result bridge remains required before B-005 completion. No compiler ABI changed.

### Original recursive value-graph admission foundation (2026-10-06)

The value-definition draft grammar and acceptance dependency are frozen separately
in value-definition-schema-bundle.json with original source hashes recorded in
value-definition-source-pins.json. The root SHA-256 is
`ae6d9227ddfe82bd531e3fb2754382f4bbe898ef20afc8690a197d78a16d62f3`.
Earlier binding and read-context snapshots remain unchanged.

The pure Rust graph admission preserves original JSON and decoded artifact bytes,
checks selected profile and accepted-definition correspondence, and validates
canonical base64/digests under explicit candidate byte/work bounds. Node IDs and
authored identities must be unique; references resolve without expanding cyclic
types. Structured nodes reference records, record field identities match their
referenced nodes, literal member names remain unique and ordered, and every node
is reachable from the root.

A positive cyclic graph test first failed against a pending implementation, then
passed. Negative controls cover dangling references, duplicate IDs/identities,
unreachable nodes and corrupted artifact digests. All 28 Rust tests pass. This
is structural graph admission, not qualification of codec meaning or complete
original UMF facet/source correspondence. Integration into candidate compilation,
operation-specific codec interpretation and native host enforcement remain work
for B-005; no adopted Truss profile or production compatibility is claimed.

### Value-graph original model correspondence (2026-10-06)

Graph source admission now checks the selected root identity and each node against
the original catalog's document/revision/module/element identity. Nodes outside
selected modules refuse. Strictly decoded authored-definition JSON must equal
the original element, including retained extension content; accepted-definition
JSON must equal the original root. Original artifact bytes remain separately
retained, so semantic JSON correspondence does not assert byte-identical source
serialization or normalize the retained artifacts.

A test uses the pinned model from the existing compiler corpus and admits its
original definition. Changed root revision, a rehashed substitute definition and
an unselected module refuse. All 29 Rust tests pass. This establishes source
correspondence only; storage shapes/facets/codec interpretation and operation
capabilities still require independent admission before using this graph for
SQL lowering. No native or embedding checks were rerun for this standalone API.

### Value-graph resolved topology correspondence (2026-10-06)

Storage graphs can now be compared with the frontend's finite resolved descriptor
graph. Admission requires the same root and exact identity closure, rejects
duplicate resolved descriptors, checks scalar family, and matches sequence/map
item references and structured-record references by authored identity. Record
member identities must match in their authored order; an otherwise valid graph
cannot silently reorder the record or substitute another type shape. Cyclic
graph comparison remains finite without schema expansion.

Tests cover a cyclic structured record, collection-shape substitution, missing
members, incomplete closure, duplicate descriptors, reordered record members and
changed scalar family. All 31 Rust tests pass. An initial test-source borrow error
was corrected before this successful run. This compares resolved logical topology
only: storage names, presence/availability, facets, representation and codec
capabilities still need their separate selected-profile admission. Candidate SQL
is not yet wired to this standalone graph API.

### Original JSONB leaf-codec selection foundation (2026-10-06)

The owner JSONB leaf-codec grammar is frozen in a separate offline closure. Its
root SHA-256 is
`ba3088c60b154de39b1985da7d283ffdbb93cb4f59c0df4d80a12462f319de77`;
leaf-codec-source-pins.json records both original schema resources. Previous
owner snapshots remain unchanged.

Static Rust admission checks the exact registered codec, source-interpretation
and native-domain profiles, then canonical base64/hash and original identity/byte
correspondence for every selected artifact. Numeric token rules additionally
require the explicitly registered original numeric-adoption evidence. Missing or
unrelated artifact selections refuse. Integer and decimal decoded-carrier kinds
must match the rule family even though the shape schema permits either spelling.
Original JSON and decoded artifact bytes remain retained under candidate bounds.

All four authored rule families are interpreted into distinct static rules. Tests
refuse rehashed source substitution, native artifact identity substitution, unknown
profiles, family changes, missing numeric evidence and coercion. All 33 Rust tests
pass. Fixture evidence remains synthetic: registration/custody does not validate
the contents of source/native/adoption artifacts, qualify a native codec, authorize
operations or establish deployment adoption. Source/facet and native procedure
interpretation, graph integration, decoding and SQL lowering remain B-005 work.

### Scalar graph-to-leaf codec correspondence (2026-10-06)

Value-graph admission can now bind the exact scalar closure to separately admitted
leaf codecs. Each scalar node requires its registered codec under its own node ID;
missing or unrelated selections refuse. Original codec artifact bytes must equal
the admitted leaf definition JSON, and the node codec profile, complete authored
artifact, family and storage representation must match that original definition.
This connects the previously separate graph and leaf admission boundaries without
allowing source or representation substitution. Compound codecs remain separate.

A positive exact selection passes. Negative controls cover a missing codec, changed
storage representation, foreign codec profile, foreign authored artifact identity
and substituted codec bytes. All 34 Rust tests pass. This is correspondence, not
source/native procedure qualification: it grants no comparison, key, predicate or
aggregate capability. Candidate SQL integration and live/native enforcement remain
required before B-005 completion.
