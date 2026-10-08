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

### Native comparator operation admission foundation (2026-10-06)

The original native-comparator grammar and acceptance dependency are frozen
separately; root SHA-256
`26f3a754456e01984841d01dd972c2051d8611b9c0f9182f48721ec6000ebd4b`
is recorded in native-comparator-source-pins.json. Static admission preserves
original JSON/artifact bytes and requires exact registered comparator/native
profiles and original identity/byte custody for value/source/native domains,
operator inventory and qualification. The caller SQL cannot supply the backend's
operation registration set.

Strategy interpretation distinguishes Unicode text/C, boolean, signed integer,
unsigned integer/numeric and finite decimal. It refuses nullable operands, scalar
family/signedness substitution, integer domains narrower than authored width,
incomplete/unknown facets and invalid precision/scale. Equality, ordering, key
and SUM registration are independent; SUM is unavailable for string/boolean even
if included erroneously in the operation set. Registered support is still a
backend assertion requiring its governing qualification evidence.

Tests cover unsigned64, signedness mismatch, int2 narrowing, rehashed inventory,
unknown native profile, missing operation registration, boolean SUM, nullable
operands, exact text/decimal strategies and unsupported facets. All 38 Rust tests
pass. These are static candidate checks with synthetic registry artifacts, not
native operator qualification, parsing/facet procedure implementation or aggregate
result-domain evidence. Connecting this admission to requested SQL operations and
qualified original profiles remains B-005 work.

### Comparator requirements from resolved query uses (2026-10-06)

The comparator requirement collector now traverses both resolved IR versions.
Relational joins/filters/grouping request equality; SUM requests aggregation on
its argument. Application requirements additionally retain ordering/cursor-prefix
equality, page-key key/order support and independently typed source/target keys
for relationship lookups and ordered RELATED_KEYS. Plain projection adds no
comparison requirement. Uses merge by complete authored identity and reject
inconsistent resolved types; key arity must match rather than silently zip away
components. The relational traversal uses explicit work stacks.

A pre-lowering admission helper requires a selected comparator for every requested
field, the exact admitted logical type/facets and each requested operation. The
comparator retains its admitted logical domain privately so a different type
cannot reuse the same operation registration. Missing comparators, missing
operation meaning and type substitution refuse.

Tests resolve the existing complete application fixture corpus through the real
frontend and collect its requirements, plus an actual relational join/group/SUM
query and projection-only query. A selected equality comparator admits the exact
field and refuses added ordering or a different type. All 41 Rust tests pass.
An initial test used a private resolver and a later stale query variable; both
were corrected before this successful run. This helper is not yet called by
candidate lowering: the synthetic candidate binding remains separate until
real profile admission is integrated. No native/embedding rerun or production
qualification is claimed for this standalone pure compiler addition.

### Selected key meaning admission in the candidate compiler (2026-10-06)

Integration inspection found an actual refusal gap: selected page and relationship
keys could replace their comparison/encoding profile or definition while fixed
candidate SQL retained its own key semantics. A regression compiled an altered
page key and failed the expected-refusal assertion before the implementation fix.
An initial test-source reference comparison error was corrected before that red
behavioral observation.

Shared candidate key admission now requires its explicit synthetic registered
comparison/encoding profiles and exact original `{}` definition bytes, ordered
property mapping and full authored key correspondence under its entity. Page
keys and both independently typed relationship endpoints use that admission.
Unselected keys are not treated as requested semantics. This refuses foreign
meaning rather than pretending the fixed candidate implements it.

The regression changes profiles and rehashed definitions for a page plus forward
and inverse RELATED_KEYS cases, mutating endpoint keys independently so a page
key failure cannot mask a target-key gap. All 42 Rust tests pass, including the
existing valid application/relational compiler corpus. Native SQL templates and
public ABIs are unchanged; native and embedding checks were not rerun for this
admission-only fix. Real selected-profile graph/codec/comparator integration and
qualified host execution remain required before B-005 is merged.

### Original native row-join selector admission foundation (2026-10-06)

The complete owner row-join grammar is frozen separately with its acceptance
dependency. Root SHA-256
`ddc7528dd546dfc76092076613fa612caee55a45ed51f46a553675b6f4e9d2b1`
is recorded in row-join-source-pins.json. Static admission checks the registered
profile, unique physical relations and columns, exact relation/name association
for every owner/state/node/scalar selector, and original identity/byte custody
for layout inventory, stored domain, all constraint evidence and conditional edge
association. Numeric-token, codec-definition and original-source columns cannot
be omitted or redirected into typed numeric columns. Original JSON/artifact bytes
remain retained under explicit candidate bounds.

The registry supplies selected physical inventory entries; their correspondence
to original inventory bytes remains a separately required procedure. This module
does not fabricate that correspondence by reading names or table tags. Closed
selector rules distinguish object ID/type from edge ID/relationship and require
edge association only for edge definitions.

Positive object and edge definition controls pass. Negative controls refuse
redirected payload columns, columns from another relation, missing original
source columns, wrong owner join, rehashed inventory and missing/contaminating
edge association. All 44 Rust tests pass. These are static mapping controls; they
do not qualify native constraints, authorized visibility or publication, and
edge definition admission does not add a logical edge-property query feature.
Candidate lowering still uses its separate synthetic mapping until real-profile
integration is completed.

### Original property-home to native-join correspondence (2026-10-06)

Binding admission now exposes original home shape/owner/property/inventory
correspondence separately from fixed synthetic candidate physical IDs. Existing
candidate property_home admission retains its original restrictions. A registered
physical definition may therefore be checked without renaming it into candidate
fixtures or granting the candidate arbitrary selector support.

The admitted row join composes with the enclosing home by exact record kind,
profile, layout inventory, state/node/scalar physical identities and original
join artifact bytes/base64/hash. Home value/presence artifacts must equal the
property artifacts. Stored-domain obligations require explicit registered IDs;
edge definitions additionally require registered association profile and exact
association definition. Registry membership does not discharge host procedures.

A complete original selector fixture with distinct physical IDs admits through
the registered join correspondence while fixed candidate lowering correctly
refuses those IDs. Unknown obligations, foreign node/profile/value and corrupted
embedded join digest refuse. All 45 Rust tests pass. Inventory-byte interpretation,
codec/native procedure qualification and actual SQL integration remain separate
B-005 work; this correspondence API grants no production compatibility.

### Recursive record-member presence correspondence (2026-10-06)

Value-graph admission now connects every record member's original presence
artifact to a separately admitted presence definition and the referenced authored
Field. Exact original presence JSON bytes and complete accepted authored artifact
identity/bytes must match. Missing, misplaced or unrelated selections refuse.
Indexed node lookup keeps member correspondence finite without repeated graph
scans or cyclic expansion.

A cyclic record admits its exact selected presence definition. Controls refuse
missing registration, foreign authored artifact identity, replaced presence bytes
and a definition registered at another member path. All 46 Rust tests pass.
This binds meaning custody only; field availability and native-null capability
remain separately interpreted, and no native absence/visibility enforcement or
SQL integration is inferred. The candidate retains synthetic definitions until
its registered real-profile admission is connected.

### Binding/home/relationship and extra-obligation refusal (2026-10-06)

The active candidate profile audit found additional unchecked selections: top-level
binding profile, selected home profile, relationship profile and declared extra
execution obligations. A red regression showed an unknown binding profile still
produced SQL. Candidate admission now requires the original explicit synthetic
binding/home/relationship profiles and refuses extra execution obligations rather
than replacing or ignoring their unknown procedures. Existing fixed context
obligations are emitted by the trusted candidate implementation, separately from
these refused input declarations.

Tests alter binding/home/obligation inputs in both original storage homes and
relationship profiles in forward props and inverse row traversals. All 48 Rust
tests pass, including the valid compiler corpus. These guards do not change SQL
templates or expand compatibility. Full original profile/procedure integration
and native host qualification remain B-005 requirements.

Fresh integration verification rebuilt the compile_probe and ran all 76 actual
PostgreSQL application cases (37 props, 39 row), including existing corruption
controls. The system Python invocation initially failed on unsupported strict zip;
the selected Python 3.12 runtime completed the full run. The owned PG17 fixture
container and synthetic storage scope remain as documented above.

The same updated core was rebuilt into the actual native PyO3 wheel and browser
WASM. Native Python passed all 76 artifact-byte parity cases with subprocesses
disabled. Chromium's initial sandbox launch failed on macOS Mach port registration;
only the browser check was rerun with escalation and passed all 76 cases, byte
parity and absence of Node globals. No build was restarted solely for a wait.
These runs exercise the updated candidate compilation path; they do not expose
the standalone real-definition APIs through wrappers or qualify Truss host duties.

The fresh browser artifact is 10,394,193 bytes with SHA-256
`d8a418d3fdcd0e5ad9147b247b1598fa099fb9731697dcbdd15efb584eb4c5c0`,
verified in Chromium 153.0.8010.12 through Playwright 1.62.1.

### Composed original property value admission (2026-10-06)

A pure property value admission gate now consumes the enclosing binding, original
selected catalog, frontend descriptors and separately registered value/presence/
leaf/record-presence meanings. It computes the exact authored closure from frontend
descriptors before inspecting the storage graph; binding-selected nodes cannot
shrink the required closure. Cycles terminate by full identity. Missing or
duplicate frontend identities refuse.

Admission checks selected profile correspondence and original owning source
document, then graph artifact custody, original authored definitions, resolved
topology, leaf codecs and all record-member presence. Root presence must preserve
the binding's full accepted artifact identity/bytes. The result retains original
graph/presence JSON and the frontend descriptor closure. This composes value gates
only; physical props/native-join and operation admission remain separate, and the
function does not claim whole-backend qualification.

A test uses an actual authored Customer.name Field from the existing pinned
compiler corpus, resolves it through the frontend, and composes full graph, leaf
and presence definitions. Empty/incompatible frontend closure and substituted
source document refuse. A cyclic closure test proves finite dependency discovery
and missing-dependency refusal. An initial test supplied an unnormalized unquoted
Name and was corrected before success. All 50 Rust tests pass. Source/native
interpretation registry payloads remain synthetic; candidate lowering has not
been broadened to these definitions, and no native/embedding run is claimed for
this standalone addition.

### Composed value and original physical-home admission (2026-10-06)

Property admission now combines the value gate with explicit registered home
selection. Original home profile and inventory identity/bytes must match. Props
selectors resolve relation/props/discriminator physical IDs to their registered
relation/name associations; object and edge discriminators differ, columns cannot
alias and member/value/presence pins stay tied to the enclosing property. Edge
association requires its original registered profile/artifact in either home.
Native rows consume their admitted original join and registered obligations; they
cannot fall back to props. Compound roots refuse scalar-root row access.

The authored props-field integration passes with independently supplied physical
metadata and refuses missing columns/changed inventory. The existing complete
original row-join fixture now passes through the composed physical-home gate,
while a row binding lacking a selected join refuses. All 51 Rust tests pass.
Registered metadata correspondence to inventory bytes, native types/procedures,
operation admission and actual SQL wiring remain explicit requirements; static
home admission cannot create these proofs. Candidate compilation remains on its
separate synthetic profile and this change adds no logical edge-property support.

### Owner-qualified comparator requirement correction (2026-10-06)

Property integration inspection found that the comparator collector's earlier
full-Field-identity merge was insufficient: the same authored Field may belong
to multiple Records with independent property homes/native domains. Requirements
now retain complete Record owner plus Field identity. Scan occurrences resolve
to their original Record before requirements merge; missing/repeated scans
refuse. Page keys use their source Record; relationship keys independently use
the resolved from/to Records, preserving inverse query roles. Registration keys
include both complete identities, and one owner's comparator cannot cover another
owner's property. This supersedes the earlier identity-only merge description.

A real frontend model test adds Customer.name as a member of Orders, resolves a
join comparing both uses and verifies distinct owner requirements for the same
Field identity. The registered comparator test additionally refuses a different
owner with the same Field/type/operation. All 52 Rust tests pass, including the
actual application corpus. Candidate SQL is unchanged: the helper is still a
pre-lowering integration boundary rather than a newly qualified target profile.
Exact property-to-comparator value/native-definition correspondence remains next
in the integration path; no physical domain is inferred from shared Field meaning.

### Original Record ownership and property-comparator correspondence (2026-10-06)

Composed property admission now resolves its physical owner catalog mapping to
the complete original Record identity. The Record and Field must belong to the
same selected document/revision cut; original Record membership must explicitly
reference that Field. Owner source and accepted Record definition must match the
original module, preserving full retained content. The admitted result retains
owner/Field identities and the complete original value-definition artifact.

Operation admission now connects owner-qualified requirements to admitted
properties and comparators. Each requested scalar must match its original
descriptor/type, complete value artifact and the leaf codec's exact native-domain
profile/definition. Separately registered comparator meanings cannot substitute
a different property artifact or native domain merely because the scalar family
is the same. Source-domain procedure semantics remain independently required.

The existing authored Customer.name integration exercises the connected gate and
refuses another owner without membership, a replaced accepted Record, missing
property selection, changed graph artifact identity and independently registered
different native-domain profile. A test-source moved-value error was corrected
before success. All 52 Rust tests pass; existing test functions gained these
controls without inflating the function count. Registry domain/evidence payloads
remain synthetic. This gate is not yet wired into public candidate lowering and
does not establish native codecs, runtime authority or profile adoption.

### Admitted home selector custody (2026-10-06)

Composed home admission retains the verified props relation, props column and
discriminator column as PostgreSQL Identifier values alongside the literal member
and owner kind. Row admission retains the complete original admitted join JSON
bytes alongside access mode and owner kind. Lowering can consume this captured
selection without fetching names from a changed registry after admission. The
original inventory and host/native qualification requirements remain unchanged.

The authored Customer.name integration checks exact quoted selectors and literal
property member. All 52 PostgreSQL crate tests pass (31 library, four binding,
17 compiler); no public candidate behavior or embedding ABI changed. Captured
selectors are an integration foundation, not emitted real-profile SQL or native
profile qualification.

### Admitted JSONB physical locator emission (2026-10-06)

Admitted props homes now emit a codec-neutral physical location from their captured
column and a separately quoted owner alias. Literal property members use typed
prepared parameters; root, JSONB leaf and text extraction remain distinct. The
locator exposes root integrity as a prerequisite and presence as unknown for
malformed roots, preserving absent versus JSON null. It performs no scalar cast,
codec interpretation, operation grant or result decoding. Thus numeric-token
profiles cannot accidentally inherit the fixed candidate's JSON-number conversion.

The authored property integration verifies quoted dotted/quoted aliases, parameter
position/value and the malformed-root presence guard. The original row-join
integration refuses JSONB lowering without allocating a parameter, preventing
row-to-props fallback. All 52 PostgreSQL crate tests pass. These are Rust emitter
checks; no new native or embedding qualification is claimed. Full registered
codec lowering, row emission and public backend integration remain open.

### Original leaf storage carrier emission (2026-10-06)

The composed property now connects its admitted root graph codec to props
extraction. Exact original codec bytes must match before allocating parameters;
compound roots refuse this scalar path. Storage extraction derives family from
those original bytes rather than a mutable parsed enum. String/integer-token/
decimal-token carriers retain extracted text; boolean conversion is guarded by
JSONB boolean type. Structural integrity combines the root prerequisite and
selected JSON storage kind. It neither casts numeric tokens nor normalizes their
spelling, and does not grant comparison or prove source/native domain validity.

The authored string-property integration checks carrier/integrity SQL and refuses
a byte-different codec before parameter allocation. All 52 PostgreSQL crate tests
pass. Numeric/boolean branch implementation is not new native qualification;
independent registered domain procedures, recursive and row lowering, result
bridge and public backend integration remain required before B-005 acceptance.

### Four-family extraction controls and native primitive probe (2026-10-06)

Leaf extraction SQL now lives with the original codec interpreter, reused by
composed property emission. A four-family Rust control changes the parsed rule
projection and verifies original-rule extraction, boolean CASE guarding and text
carriers for Unicode/integer/decimal. All 53 crate tests pass (32 library, four
binding, 17 compiler).

The independent leaf-storage-primitives.sql probe ran through prepared text member
selection on the owned PostgreSQL 17.9 container. Ten rows returned the expected
physical observations: é😀 with two trailing spaces retained; false as native
boolean false; 18446744073709551615 and -0.00 retained as exact string tokens;
a JSON number retained its number kind; invalid boolean text never entered the
boolean cast. Missing returned present=false; JSON null returned present=true
and kind=null; array and SQL-null roots returned unknown presence. This verifies
native extraction primitives, not compiler-generated full queries, token-domain
admission, complete-result publication, deployed codec adoption or native profile
qualification. Reproduce with psql -X -q --csv -v ON_ERROR_STOP=1 against the owned
fixture container using tests/truss-postgresql/leaf-storage-primitives.sql.

### Captured native row root location emission (2026-10-06)

Composed properties now retain their admitted owner/property catalog IDs and
expose row-root location emission through the captured original join bytes.
Props homes refuse this path. State/root-node/scalar LEFT JOINs preserve the
original registered object or edge owner discriminator and property owner pair;
namespace, aliases and relation names are separately quoted. Catalog IDs are
signed int4 prepared parameters. Location emission is physical access only and
does not assert node/payload uniqueness, stored-domain validity, authorization
or qualified whole-tree/result decoding.

Both catalog domains validate before allocation, and the complete location uses
staged parameter custody: a second-slot overflow leaves the original collection
unchanged. Object/edge definition fixtures check distinct owner ID/discriminator
columns, dotted namespace quoting, ordered signed catalog values and a 1023-slot
overflow refusal. All 54 PostgreSQL crate tests pass (33 library, four binding,
17 compiler). These are emitter/component controls, without new native execution
or public wrapper qualification. Logical edge-property queries remain separately
unfinished; a physical edge join foundation does not introduce their frontend syntax.

### Row root structural prerequisites and alias containment (2026-10-06)

Captured row location emission now returns structural_integrity separately from
its LEFT JOINs. Correlated independent counts require either complete state
absence or exactly one state with one parentless existing root, and at most one
root scalar payload. Counts use the full original owner-kind/id/discriminator/
property-owner/property tuple; missing roots and duplicated state/root/payload
rows cannot be suppressed by inserting integrity as a query filter. Required
versus optional state presence, scalar versus compound payload requirements,
source/domain/codec correspondence and complete subtree checks remain separate.
The host must evaluate prerequisites in the same complete authorized view before
logical operations or publication; this predicate does not establish visibility.

Internal join and probe aliases reject collision with the supplied owner alias
before parameter mutation. Object/edge component controls check the prerequisite
count/root structure and probe-alias refusal, preserving atomic slot custody.
All 54 PostgreSQL crate tests pass. This turn adds no native corruption result or
profile qualification; native execution of emitted prerequisites remains required.

### Native execution of emitted row structural prerequisites (2026-10-06)

The original object/edge row-location Rust test now supports explicit test-only
emission capture through WEFT_ROW_LOCATION_CAPTURE. Each definition fixture
passes its original registration gate; emitted joins/prerequisite bytes are
captured directly rather than rebuilt in the host. The saved capture is
tests/truss-postgresql/fixtures/row-location-emission.json. The independent Python
harness row-location-native.py executes those expressions with unchanged prepared
int catalog parameters against rollback-isolated unconstrained fixture tables.

Twenty scenarios (ten per owner kind) pass on PostgreSQL 17.9. Duplicate state,
root and payload rows produce false prerequisites on every observed duplicate;
missing/parented roots and NULL state IDs refuse. Independent absence, valid
roots and compound roots without scalar payload pass the structural subset.
Foreign owner state remains absent for the selected full tuple; this is not proof
of authorization visibility. Exact scenario membership/counts are independently
authored. The JSON receipt records original capture, generated SQL and harness
hashes plus the actual engine version and all observations. No full Truss native
profile, recursive codec, typed payload domain or publication gate is qualified.

Reproduce capture with cargo test -p weft-postgresql --locked root_locations_keep
and WEFT_ROW_LOCATION_CAPTURE set to an explicit file; run the Python harness
with that file against owned weft-b005-pg17. All 54 crate tests also pass without
the capture environment. The twenty executions are separate physical-component
evidence, not additional application story acceptances or production claims.

### Native scalar observation custody (2026-10-06)

Captured row-root locations now expose raw scalar observations for the registered
decoder: payload presence/kind, native text and boolean, numeric_value rendered
as text, independent original numeric_token, and codec/source bytea rendered as
lossless hex. The four unrelated binary/temporal/opaque slots remain visible
through an explicit all-NULL prerequisite for the selected four-family subset.
This projection does not decode native temporal or opaque content, expand scalar
support, enforce typed-slot exclusivity or establish source/native correspondence.
The original numeric token is never replaced by the native numeric rendering.

The original object/edge location controls verify independent token/native-text
expressions, codec/source custody and unrelated-slot checks. All 54 crate tests
pass. No new native observation execution is claimed; decoding, exact source
grammar/facets, source/native equality, codec-byte identity and final publication
remain separately required by Truss CONTRACT-010's row scalar matrix.

### Typed row slot prerequisites and native custody controls (2026-10-06)

Scalar observations now emit explicit four-family physical payload prerequisites:
selected scalar tag/payload presence; unrelated payload slots NULL; required
codec/source bytes present; numeric families retain both native numeric text
and original token. These checks grant no source grammar, facet, comparison or
source/native equality capability. Root/subtree integrity and original codec-byte
identity remain independently required.

WEFT_ROW_PAYLOAD_CAPTURE records exact Rust-emitted expressions from the existing
original object/edge definition fixtures. row-payload-native.py independently
constructs 68 cases on PostgreSQL 17.9: valid carriers, absent selected columns,
wrong token slots, wrong kind, mixed payloads, absent codec/source and present
unrelated binary bytes. Exact byte hex and numeric/token custody agree with the
authored oracle. Four numeric value/token mismatch cases deliberately satisfy
physical slot structure, proving that the separate source/native semantic check
is still mandatory. These must never be treated as publishable logical values.

The original capture and native JSON receipt preserve expression/harness/generated
SQL hashes, exact engine version and all observations. Reproduce the crate tests
with WEFT_ROW_PAYLOAD_CAPTURE set to a file, then run row-payload-native.py with
that capture and explicit output receipt path against owned weft-b005-pg17.
A test-only Family moved-value error was corrected before execution. All 54
PostgreSQL crate tests pass. This is native component evidence, not acceptance
of a deployed Truss profile or an application/story support claim.

### Bounded exact mathematical correspondence witness (2026-10-06)

The numeric_correspondence Rust module realizes Truss CONTRACT-010 NX's compact
mathematical witness over separately admitted sign/mantissa/fraction/exponent
parts. It removes leading/trailing coefficient zeros with checked signed exponent
arithmetic, retaining original caller-owned tokens untouched. Zero cannot bypass
complete digit/counter checks. No power-of-ten expansion or floating arithmetic
is used. An invocation-shared budget reserves four steps per digit plus fixed
bookkeeping for each occurrence, including repeated equal values; caller-selected
maximum digits bounds witness allocation. This candidate reservation convention
is not adoption of Truss's complete value/resource profile.

The original Truss draft mathematical vectors are copied with a source SHA-256
pin checked by the Rust test. All eleven witnesses and seven equality pairs pass,
including >2^53 adjacent integers, negative zero, spelling variants and bounded
million-exponent algorithm controls. Overflow on zero, malformed parts, inconsistent
fraction counts and repeated budget exhaustion refuse. Full crate verification
passes 56 tests (35 library, four binding, 17 compiler); the added source-pin check
then passes targeted mathematical tests.

The helper consumes already parsed parts and grants no source/native grammar,
facets, native representability, ordering/key/SUM or publication capability.
Source and native parsers, full original definition/domain admission and source
custody must precede witness equality. Million-exponent mathematics is not a
PostgreSQL storage claim. Decoder/public backend integration and deployed native
profile qualification remain open.

### Original property admission bound to backend invocation context (2026-10-06)

Every composed property now privately retains SHA-256 of its complete original
binding bytes. verify_binding_basis refuses reuse under another binding cut.
The new comparator_requirements::admit_context gate accepts the shared backend
Context, verifies its raw binding checksum/decoded original, checks owner-qualified
registration keys and every property's exact original basis, requires admitted
properties for selected fields, and collects/admit-checks requested operations
against their coupled original properties/comparators. Coverage of compound
descriptors, relationships, capabilities and host obligations remains separately
required by the core/backend contract; this helper is not the finished backend.

The authored Customer.name integration constructs an actual resolved projection
Context and accepts the exact original basis. Missing selected property and
a schema-admitted namespace change with a fresh correct checksum refuse before
lowering, even though the selected property graph remains unchanged. Direct basis
verification also refuses foreign bytes. The full crate suite passes 56 tests;
the strengthened valid namespace-control then passes its targeted integration.
The public candidate still consumes only its fixed synthetic profile; registered
context-gate invocation by a complete backend and wrapper qualification remain open.

### Owner-qualified read coverage independent of comparator grants (2026-10-06)

collect_reads now records original owning Record/Field tuples for every relational
field expression and application projection, including compound outputs, plus
fields required by predicates/keys/relationships/aggregates. It reuses the resolved
plan and retains independent owners of a shared authored Field. Context admission
requires each exact tuple's property registration before operation-specific
comparator admission. Identity-only selected-field coverage can no longer stand
in for the correct owning property's admission. Plain projection does not require
or grant equality/order/key/SUM merely to read a stored value.

The 76-case application corpus checks every projected field and requested
operation's tuple appears in read coverage. Relational controls verify one plain
projection with zero comparator requirements and a shared Field under two owners
with distinct read admissions. The composed original-property Context accepts
plain projection with an empty comparator registry; its targeted integration
passes after the full 56-test crate suite. Registered full-backend invocation,
recursive result decoding and native/profile/host qualification remain unfinished.

### Context-admitted physical access planner (2026-10-06)

registered_access::lower now connects the complete context/property/comparator
gate to captured props and native-row location emission. Backend-owned access
requests name original scan occurrences and Fields. The planner resolves owning
Records from the original plan, checks selected owning tuples, derives distinct
quoted scan aliases and the exact admitted namespace, and stages all parameters
for the entire request set. Unknown scans, foreign read tuples and duplicate
access requests refuse before any staged slots are committed. Repeated reads
under distinct self-join occurrences retain separate aliases/locations.

The authored-property integration lowers an actual resolved projection through
this path without comparator grants. A late foreign scan request after a valid
access leaves the original parameter collection empty. A real resolved Customer
self-join accepts the coupled equality comparator and emits two distinct scan
aliases/two member slots. The full 56-test suite passes; the added self-join
integration then passes targeted verification. These tests still use synthetic
registered domain evidence.

This is a physical access stage: the backend still owns selecting requests from
resolved expressions, full relational operations, original codec/result decoding
and complete prerequisite/publication protocol. Independent relationship subquery
accesses and full recursive storage semantics remain separate work. No public
compiler/backend support or native Truss profile qualification is claimed.

### Resolved expression-to-access request generation (2026-10-06)

registered_access::requests derives exact outer scan/Field pairs from relational
expressions and application projections, predicates, grouping/ordering, SUM,
authored page keys and relationship source keys. Repeated references deduplicate
only within the same occurrence/Field tuple. lower_plan feeds those requests
through the admitted context and atomic physical planner. Explicit request
lowering now checks exact occurrence/Field membership rather than only owning
Record/Field membership, preventing a shared Record's field from moving to an
unreferenced self-join side. Relationship target subqueries remain separately
scoped access plans; this is not complete relationship query emission.

The real Customer self-join lowers automatically to two locations/member slots.
A different self-join projects name only from its first occurrence; the independent
control checks three total accessed pairs and absence of name on the second
occurrence. All 76 application cases verify projected scan/Field pairs appear
in generated requests. A function-name shadowing compile error was corrected
before successful verification. All 57 crate tests pass (36 library, four binding,
17 compiler), with documentation checks also passing. Original codec/relational
result integration, target-subquery lowering and native/profile/host qualification
remain required for B-005 acceptance.

### Captured codec extraction attached to access planning (2026-10-06)

Value admission privately captures the already admitted leaf-codec closure.
Registered physical access planning now attaches scalar props carrier and
storage-kind expressions using those captured original definitions, with exact
codec-byte correspondence against the admitted graph. No later registry lookup
or caller-provided family chooses extraction. Compound roots return no scalar
template and retain their existing graph for recursive lowering; native row
source/native decoding remains separate rather than reusing JSONB codecs.

props_node_storage exposes the same exact captured lookup by original graph
node index for recursive decoder composition, refusing an out-of-graph index.
Root extraction delegates to it. Slot/presence/path correspondence, source/native
grammars/domains and public result decoding remain independent prerequisites;
no scalar template is evidence of complete-value admission or publication.

The original projection Context now produces its quoted scan/member carrier and
string storage check through the complete access planner. Original node lookup
agrees with root extraction, and an invalid node index refuses. Full crate tests
pass 57; node-lookup refinements then pass their targeted integration. These are
component tests with synthetic registry domain evidence. Full recursive/row
result integration and deployed Truss qualification still gate B-005 acceptance.

### Original owned-property presence observation (2026-10-06)

Property admission now exposes props presence observation through its admitted
literal member and original presence definition. The root Field descriptor's
required/absent-allowed availability controls absence; an absent required Field
is an obligation refusal. Scalar authored nullability remains distinct from
availability, and explicit JSON null still requires separately qualified native
null semantics. Compound native null is not inferred from field availability.
Native row homes refuse this JSONB procedure and need their separately selected
row presence rules.

The original required Customer.name fixture accepts empty text as present and
refuses missing member, explicit JSON null, array root and SQL-null root. All
57 PostgreSQL crate tests pass. A present raw JSON value still requires exact
codec/source/native/type admission before publication; this observer does not
qualify authorized visibility, native-null support or complete result decoding.
Full backend/result integration and Truss profile qualification remain open.

### Original finite decoder slot layout attached to access plans (2026-10-06)

Admitted graphs now expose a borrowed finite layout for recursive decoder
composition: scalar family/storage representation, sequence/map item indices,
structured Record index, and ordered Record slots with full Field identity,
literal stored member name, referenced value index and original presence bytes.
Every node retains original codec bytes. Stored names are never replaced by
authored display names or interpreted as path syntax. Cycles remain finite
node references rather than expanded recursive schemas. Codec artifact custody
does not itself interpret or qualify a compound codec.

Registered accesses now carry this layout alongside physical location and scalar
props extraction. Within one admitted owning property, self-join occurrences
share an immutable layout allocation while keeping independent aliases/slots.
Layout sharing does not deduplicate runtime value validation or reset resource
charges; bytes remain borrowed from the admitted original graph.

The cyclic structured/Record fixture checks its back-reference and original
presence bytes. A literal name containing a numeric-looking prefix, dot, brackets
and quote remains exact. The actual property projection checks attached root/node
metadata; its self-join checks shared layout identity. All 57 crate tests pass.
Recursive walk/source/domain/result realization, native whole-value probes and
full backend/profile qualification remain unfinished; this metadata view is not
a public support or publication claim.

### Backend-owned native expression traversal (2026-10-06)

A shared iterative expression renderer now passes original typed nodes and
ordered rendered operands to a trusted backend callback. Field location, literal
conversion and every native operator remain callback-owned; no model content
loads executable code or selects an unregistered SQL template. The candidate
uses this traversal with its existing fixed profile meanings. The original-profile
backend can supply independently admitted mappings/operators without inheriting
JSONB assumptions or the candidate's literal casts. Whole-expression parameters
commit only after every callback succeeds.

An actual resolved equality control verifies left-to-right callback order and
ordered native operands. A callback allocates the first property slot then
refuses a later missing native meaning; original parameters remain unchanged.
All 58 crate tests pass (37 library, four binding, 17 compiler). The rebuilt actual
compile_probe passes all 76 original application/native cases (37 props/39 row),
including the existing corruption/empty/order controls. The JSON receipt pins
new renderer/candidate source, native harness/corpus, probe binary, full output
report and actual PostgreSQL 17.9 version. Python/browser packages were not
rebuilt this turn; their earlier pinned component evidence remains separate.

The traversal grants no native grammar/domain/operator authority. Original
registered procedure integration, complete result bridge and native Truss
profile/host qualification remain open before B-005 acceptance.

### Backend-owned relational source assembly (2026-10-06)

The V01 candidate source emitter now uses iterative relational assembly with
trusted scan and native-expression callbacks. It retains left/right occurrence
order, filter placement and grouping, while staging all source parameters until
successful completion. Integrity prerequisites remain separate from filters.
An explicit aggregate marker also catches global aggregates with no grouping
columns: joins or filters over an aggregate, repeated aggregation and nested
projection refuse until a separate target stage is implemented. Group-list
emptiness cannot stand in for aggregate-stage identity.

Three controls use actual resolved plans: grouped join source/group ordering,
late join refusal after both scan callbacks allocate parameters, and global
aggregate refusal beneath a join or filter. All 61 crate tests pass (40 library,
four binding, 17 compiler). A freshly rebuilt compile_probe passes the original
26 relational/native cases across props and row homes against PostgreSQL 17.9.
[Receipt](B-005-relational-native.json) pins source, harness, corpus, binary and
full report. These V01 cases directly exercise the source/expression refactor;
the preceding 76 application cases use the separate V02 application emitter.
Python/browser packages were not rebuilt for this change.

This is fixed synthetic candidate evidence. Original-profile integration,
complete result bridge and adopted Truss native/host qualification remain open;
no story criterion or B-005 merge gate is closed by these component checks.

### Selected access-to-expression bridge (2026-10-06)

The original-profile preparation API now connects typed expression traversal to
its occurrence-qualified admitted physical access list. Field callbacks receive
the exact selected access; operator/literal callbacks receive no field location.
A missing occurrence/field pair or duplicate access refuses rather than borrowing
another self-join location. Backend callbacks continue to own selected codec,
literal and native operator meaning; this bridge does not grant their authority
or publish a decoded result.

The existing original-property admission control now renders the actual resolved
self-join equality through both automatically prepared accesses. It checks the
two independent quoted aliases and captured scalar carriers. A second invocation
with only the first occurrence allocates a parameter through the first native
callback, then refuses the missing second occurrence and retains zero parameters.
All 61 crate tests pass. [Receipt](B-005-selected-expression.json) pins the bridge,
extended control and test log. No database or embedding rerun is claimed for this
standalone original-profile preparation API. Public registered backend/result
integration and original native Truss profile qualification remain pending.

### Complete lexicographic comparator operands (2026-10-06)

Original-profile comparator collection now requires both equality and ordering
for field-valued right operands of lexicographic comparison, retaining each
field's original record owner. Previously only left columns were collected.
Empty or unequal tuple arity and unequal original logical types refuse before
admission; no truncated zip or family-only comparison is accepted.

A regression control starts with an actual resolved Customer/Orders count join,
then constructs a programmatic lexicographic predicate using those two original
field operands. Both owner-qualified requirements include equality/ordering and
are covered by read admission. Empty, unequal-length and mismatched-type controls
refuse. The programmatic predicate is a collector control, not a new SQL syntax
or execution support claim. All 62 crate tests pass (41 library, four binding,
17 compiler), including original application-corpus requirement collection.
[Receipt](B-005-comparator-operands.json) pins changed source and the test log.
No native engine or embedding rerun is claimed; original-profile integration
and native Truss qualification remain open.

### Original owner-source access assembly (2026-10-06)

Admitted props and row homes now retain their owner relation and discriminator
mapping. Registered access planning emits a separately quoted namespace/relation
and occurrence alias, with an int4-domain catalog parameter for owner selection.
Each occurrence shares one immutable source across its selected properties;
conflicting physical owner mappings or catalog IDs refuse the complete access
plan. Separate self-join occurrences retain separate source parameters. Catalog
discriminators remain storage selectors, never logical key values.

Existing original-property controls now check the props owner source and its
parameterized discriminator together with exact scalar carrier extraction; the
self-join prepares two independent source/member pairs. The admitted original
row-home control checks literal dotted namespace and quoted alias spelling,
negative historical catalog IDs, and canonical/range/injection refusals with
no parameter leakage. All 62 crate tests pass. [Receipt](B-005-owner-source.json)
pins changed source and the test log. No native database or embedding rerun is
claimed for this standalone preparation change.

The discriminator selects owner types and cannot replace integrity prerequisites
or hide malformed selected properties. This API prepares scans with selected
property accesses; fieldless count scans still require independently admitted
record mapping. Edge association, recursive result decoding, original source
codec/native operation realization and complete host/profile qualification
remain independent obligations before public backend integration/acceptance.

### Admitted physical scans in relational assembly (2026-10-06)

The registered access layer now assembles each original V01 scan from its exact
owner-qualified accesses. Native row-root LEFT JOINs remain attached to that
owner source. Props root and row structural prerequisites are returned separately;
they never enter query filters. The relational assembler accepts backend-owned
scan filters, preserving owner discriminator selection across join/filter/group
stages. Existing candidate string-scan emission uses a compatibility wrapper.
Missing fieldless mapping, foreign original owner, disagreeing source/alias and
duplicate field accesses refuse before returning a physical scan.

The original-property self-join control now composes admitted sources and exact
expression accesses through the relational assembler. It checks both owner
filters and separate root prerequisites, and refuses missing and changed-owner
scans. All 62 crate tests pass. An optional WEFT_OWNER_SCAN_CAPTURE writes the
actual emitted source query and unchanged parameters from this control.
[Captured fixture](../../../../tests/truss-postgresql/fixtures/owner-scan-emission.json)
is executed by [native harness](../../../../tests/truss-postgresql/owner-scan-native.py)
against independent minimal owner rows on PostgreSQL 17.9. Empty input yields
zero rows; duplicate/Unicode/trailing-space/empty text yields exactly seven bag
rows. Unrelated owner rows, including an unrelated malformed root, do not
participate. Expected pairs are independently enumerated from fixture names.
[Receipt](B-005-owner-scan-native.json) pins capture, harness, source and actual
native results. The freshly rebuilt candidate probe also passes all 26 original
relational/native cases after the shared assembler change.

These two new executions establish physical source assembly only. They do not
execute the retained integrity prerequisites, qualify original comparator/codec
procedures, decode the public result ABI or adopt a Truss deployment profile.
Fieldless record mapping, complete recursive result bridge, original-profile
public backend and full native/host qualification remain open. Python/browser
packages were not rebuilt; no story acceptance or B-005 merge gate is closed.

### Owner-wide structural preflight queries (2026-10-06)

Physical scan preparation now returns executable count-of-violation SQL for
its separately retained structural prerequisites. Each check covers the complete
selected owner source and property joins, using only the original owner-type
selector. Query filters, joins to other logical scans, grouping and limits never
restrict this scope. IS DISTINCT FROM TRUE treats unknown/SQL NULL checks as
violations. Hosts must establish zero violations before query execution and
publication in the same admitted complete visibility/transaction context; these
checks do not create that context or authorize an incomplete RLS view.

The actual original-property preparation control captures both occurrence-wide
checks with the self-join query and unchanged parameters. All 62 crate tests pass.
[Native harness](../../../../tests/truss-postgresql/owner-preflight-native.py)
executes the [captured SQL](../../../../tests/truss-postgresql/fixtures/owner-scan-preflight-emission.json)
on PostgreSQL 17.9. Six independent fixtures cover empty and valid seven-row bags,
then selected array, JSON null, number and SQL NULL roots. Each corrupt root gives
one violation for each occurrence; the fixture does not execute the result query.
An unrelated malformed owner root contributes no violation. These selected bad
roots have no usable name for the result join, demonstrating why owner-wide checks
must precede query selection. [Receipt](B-005-owner-preflight-native.json) pins
source/capture/harness and exact native observations.

This evidence qualifies only structural check emission/execution over synthetic
props roots. Source/native codec and payload-domain checks, required/optional
presence, recursive decoding, complete visibility enforcement, general host
publication gating and adopted Truss profile integration remain separate. Native
row structural check query composition is emitted but not exercised by these six
fixtures. No Python/browser package rebuild or B-005 acceptance claim is made.

### Owner-wide native row structural preflights (2026-10-06)

Physical location composition is shared between original scan access planning
and the original row-join controls. The latter now capture complete object/edge
owner sources, original LEFT JOIN chains and owner-wide structural violation
queries using three unchanged catalog parameters. Owner names derive from the
validated original selector inventory; full property/context/codec admission
remains a separate boundary. All 62 crate tests pass.

[Native harness](../../../../tests/truss-postgresql/row-scan-native.py) executes
[actual captured Rust queries](../../../../tests/truss-postgresql/fixtures/row-scan-emission.json)
against minimal unconstrained PostgreSQL 17.9 tables. Twenty cases cover both
physical owner kinds with absent/valid state, missing root, duplicate state/root/
scalar, parented root, null state ID, wrong owner and compound root. Every valid
case gives zero violations; corrupt cases give the independently specified one
or two violations according to actual LEFT JOIN multiplicity. Wrong-type owner
rows are also installed and excluded solely by the original owner discriminator.
[Receipt](B-005-row-scan-native.json) pins capture, generated native SQL, harness,
source and exact observed counts. Transactions roll back all synthetic objects.

These checks establish structural query composition/execution only. Absence and
compound roots deliberately pass these structural controls without proving
required/optional presence or complete value validity. Row payload/source/native
codec checks, complete visibility, host publication gating and adopted native
Truss qualification remain open. Edge physical fixtures do not grant logical
edge property selection. Python/browser packages were not rebuilt, and B-005
acceptance remains pending.

### Property-independent original Record sources (2026-10-06)

A separate Record admission now verifies the original selected UMF Record,
complete source document and accepted entity definition against the binding.
Trusted registry selectors must name an object relation and type_id column in
the exact original layout inventory. Private admitted custody retains the model
pin and binding cut. Scan lowering supports V01/V02 original scan occurrences,
including fieldless counts, with derived aliases and typed catalog parameters;
duplicate occurrences, missing records or changed owner/model/binding refuse
atomically. Logical edge association does not follow from an entity type ID.

The regression control resolves an actual Customer count with zero selected
property reads and emits its independent owner source. A two-source count join
with only its first Record admitted refuses after source preparation and leaves
zero parameters. Changed namespace/binding, changed model pin, redirected native
column and mismatched original entity source also refuse. All 63 crate tests
pass (42 library, four binding, 17 compiler).

[Native harness](../../../../tests/truss-postgresql/record-count-native.py) executes
[captured Rust source/count SQL](../../../../tests/truss-postgresql/fixtures/record-count-emission.json)
on PostgreSQL 17.9 using an independent minimal object table with only id/type_id
and no property columns. Empty and two-owned-row cases return exact 0/2 counts,
excluding an unrelated owner type. Catalog IDs are negative historical values;
object IDs differ from their discriminator and have no business-key meaning.
[Receipt](B-005-record-count-native.json) pins source, capture, harness and native
observations. This closes the preparation API's fieldless Record mapping gap;
it does not finish the public original-profile backend.

Inventory interpretation and native storage guarantees still require registered
owner qualification. Property/value/comparator admission, host authorization,
recursive decoding/result publication and adopted Truss profile remain separate
before B-005 acceptance. The captured count is a source component control, not
full public compiler/result ABI execution. Python/browser packages were not
rebuilt, and no story criterion or merge gate is closed.

### Atomic combined Record/property preparation (2026-10-06)

One preparation API now admits every independent Record source, checks selected
property homes against that original owner/catalog/mapping/binding cut, and
prepares field accesses using the Record sources. Self-join occurrences share
their own source across properties; no second owner-source parameter is allocated.
Fieldless count scans are retained with no property access or inferred property
integrity checks. Structural prerequisites remain owner-wide and separate from
result selection. All parameters commit only after Record, property and comparator
admission and source assembly succeed.

Extended controls check projection source reuse, two self-join source/member
pairs, mismatched owning Record, fieldless count preparation and comparator
refusal after both Record sources are staged with zero parameter leakage.
All 63 crate tests pass. An optional WEFT_COMBINED_CAPTURE emits the actual
combined self-join query, owner-wide checks and reordered unchanged slots.
[Captured fixture](../../../../tests/truss-postgresql/fixtures/combined-preparation-emission.json)
passes all six PostgreSQL 17.9 cases through the
[native harness](../../../../tests/truss-postgresql/owner-preflight-native.py).
The harness derives int4/text prepare domains from original parameter origins
and checks their logical families, preserving both previous and new allocation
orders. Empty/valid exact bags pass; array/JSON-null/number/SQL-null owned roots
are detected before the fixture runs its result query. [Receipt](B-005-combined-preparation-native.json)
pins all changed preparation sources, capture, harness and native observations.

This combines preparation components, not the full public backend/result bridge.
The native self-join callback remains the explicit synthetic string-equality
control. Source/native domain and recursive decoder realization, native visibility,
full host enforcement and adopted Truss profile qualification remain open before
B-005 acceptance. Python/browser packages were not rebuilt; no story criterion
or final merge gate is closed.

### Original property result contract bridge (2026-10-06)

A result metadata bridge now constructs the existing Weft Column representation
from the admitted original property descriptor. Required non-null scalars retain
complete authored logical types/facets with text carriers and their text/boolean/
exact-integer/exact-decimal decoders. Optional, nullable and compound values retain
the original descriptor identity with nativeNull=false, preserving presence/value
semantics for the required result bridge. Source identity, output position/name
and public column nullability are explicit. Unestablished property availability,
invalid output identity and a graph changed from original admitted bytes refuse.

Original descriptors are now exposed as an immutable slice outside this crate;
backend preparation retains the admitted closure rather than permitting a caller
to rewrite result facets in place. The actual admitted string-property control
checks its scalar/text result contract. Two additional controls cover all four
families with exact u64 and decimal facets, optional/nullable transitions,
sequence/map/structured identities, finite self-references, missing availability
and invalid position/name. All 65 crate tests pass (44 library, four binding,
17 compiler). [Receipt](B-005-result-contract.json) pins source and the test log.
No database or embedding rerun is claimed for this metadata-only component.

These instructions state the required public result shape, not successful codec
execution. The backend still must establish the selected source/native codec and
presence conversion, emit the correct carrier, enforce native/host obligations
and withhold all results until complete admission. No new public decoder ABI,
Truss carrier substitution, native conversion claim, story acceptance or B-005
merge-gate completion follows from constructing Column metadata.

### Projection-order result contract assembly (2026-10-06)

The result bridge now assembles Column metadata in original V01/V02 projection
order after the context/property/comparator gate. Direct Field outputs resolve
their exact owning scan and original property; V01 Field types must match the
admitted original scalar descriptor. Computed scalar/aggregate metadata preserves
resolved logical types, nullable outcomes and contributing source identities.
Application counts and related-key outputs retain the existing record/key and
relationship metadata conventions. No backend-native operation is inferred from
constructing these representations.

Extended original-property controls verify separate self-join output positions
and aliases despite equal Field source identities, reject missing comparator
registration and altered projected Field type, and verify the fieldless count's
exact-integer representation. An additional scalar aggregate metadata control
preserves nullable decimal output and exact precision/scale. All 66 crate tests
pass (45 library, four binding, 17 compiler).

The combined Rust capture now includes actual emitted columns with the original
query/checks/parameters. [Fixture](../../../../tests/truss-postgresql/fixtures/projection-contract-emission.json)
passes six PostgreSQL 17.9 cases via the
[native harness](../../../../tests/truss-postgresql/owner-preflight-native.py),
which checks successful native headers against captured column positions/names
and scalar/text decoder metadata. Exact duplicate/Unicode/empty-string bags pass;
corrupt roots are detected before result execution. [Receipt](B-005-projection-contract-native.json)
pins changed sources, capture, harness and native observations. This exercises
string projection metadata correspondence only; aggregate/numeric/compound and
related-key metadata are not newly qualified by these native cases.

Full public artifact/decoder registration, executable original codec conversion,
recursive value admission, qualified host observation/publication and adopted
Truss profile remain open. No Python/browser rebuild, complete result ABI claim,
story acceptance or B-005 merge-gate completion is made.

### Scalar projection with owner-wide payload preflight (2026-10-06)

The result bridge now emits a required non-null props scalar as the existing
text-carrier column, recomputing extraction from its captured original codec.
Each projection retains original codec/presence bytes and emits a separate
owner-wide physical payload violation query. This query uses only the original
owner source/discriminator; result joins/filters cannot restrict its observation.
Wrong payload kinds and absent required scalar slots are not converted to a
successful SQL NULL result. Numeric-token extraction remains physical text;
selected source grammar/facets and native domain admission remain mandatory.

Prepared accesses now retain their private original binding checksum. Result
emission verifies that checksum plus original owner/Field before coupling access
to a property. A second valid property admission from a changed namespace/binding
cut is refused against the earlier access despite equal logical identity.
Optional/nullable/compound value and native row result paths explicitly require
their own selected bridge; they cannot reuse the props scalar template.

All 66 crate tests pass. [Native harness](../../../../tests/truss-postgresql/scalar-projection-native.py)
executes [fresh Rust-captured projection/preflight SQL](../../../../tests/truss-postgresql/fixtures/scalar-projection-emission.json)
on PostgreSQL 17.9. Nine cases cover empty and exact duplicate/Unicode/trailing-
space/empty-string output, followed by required absence, JSON null, boolean,
number, array, object and malformed-root violations. Each invalid owned fixture
has one violation and no result query is executed; an unrelated owner's wrong
payload contributes no violation. [Receipt](B-005-scalar-projection-native.json)
pins sources, capture, harness and exact native observations.

This is required string projection and physical payload evidence on synthetic
props rows. Original domain/codec/presence qualification, general publication
and coherent native observation, numeric/row/recursive result realization and
adopted Truss profile remain open. Python/browser packages were not rebuilt;
no complete public decoder, story acceptance or B-005 merge-gate claim is made.

### Prepared-read payload collection (2026-10-06)

`result_definition::read_payload_observations` collects owner-wide payload SQL and original codec/presence bytes for every prepared access, independently of projected output membership. This enables later SELECT assembly to retain checks for predicate/join/group/aggregate reads. Repeated occurrence/property entries and missing original property registrations refuse; each entry rechecks private binding custody through `property_projection`. The existing admitted self-join fixture checks both occurrences, retained originals and missing-registration refusal. All 66 PostgreSQL crate tests passed (`cargo test -p weft-postgresql`; local log `/private/tmp/weft-read-payload-tests.log`). No new native execution occurred. Required non-null scalar JSONB roots are the current executable bridge subset; native row, optional/null and recursive bridges still refuse and remain required B-005 work. Physical payload observations do not prove source grammar, facets, numeric correspondence, complete visibility, coherent execution or publication authority. B-005 remains unqualified and PR #7 remains draft.

### Original-profile SELECT assembly (2026-10-06)

`select_definition::assemble` connects prepared Record/property sources, relational staging, backend-owned expression callbacks, existing scalar output metadata and complete prepared-read physical checks. Preparation pins the exact serialized logical plan, binding digest and ordered parameter slots; stale context or changed parameter custody refuses before callbacks. New parameter slots commit only after complete assembly. Checks remain separate from logical filters and output SQL. Every prepared scan must belong to the query inventory. The admitted self-join test checks source filters, both output columns, both payload checks and structural checks; stale-plan and callback-after-allocation refusals leave original parameters unchanged. All 66 PostgreSQL crate tests passed, with fresh capture from the final source (`WEFT_SELECT_CAPTURE`; log `/private/tmp/weft-select-tests.log`).

[Native receipt](B-005-select-assembly-native.json) records PostgreSQL 17.9 execution of the [captured SELECT](../../../../tests/truss-postgresql/fixtures/select-assembly-emission.json): empty bag, exact seven-row duplicate/Unicode/empty-string self-join bag and owner-wide boolean-payload corruption detected before the harness executes the query. The harness runs all captured structural/payload count queries and withholds publication/query in the corruption case. Receipt pins sources, capture, harness and test log. This is minimal synthetic string-JSONB evidence with a fixture-native equality callback, not production operator, source/domain, native-row, recursive result, installed Truss profile or host qualification. V02 assembly and unsupported result bridges explicitly refuse here; existing application emitter is unchanged. Public original-profile backend integration and the remaining B-005 gates stay open.

### Codec-owned direct SELECT projection (2026-10-06)

Direct Field outputs in original-profile SELECT assembly now use `property_projection`, recomputing the carrier from original admitted codec bytes and comparing the complete resulting Column metadata with context-selected output metadata. Direct projections no longer ask the operator callback for a result carrier. The synthetic self-join fixture confirms only its two join operands reach that callback. A changed ordered parameter inventory refuses before callback execution. All 66 PostgreSQL crate tests passed (`/private/tmp/weft-select-codec-tests.log`); fresh final-source capture and all three native SELECT/preflight cases passed again. The native receipt was regenerated with current source and log pins. This does not qualify computed-output source/domain procedures, row-native codecs, recursive results or the host execution/publication bridge; the same B-005 gates remain open.

### Native scalar decoder custody projection (2026-10-06)

`ScalarObservation::custody_projection` emits eight private observation columns for later selected row decoding: scalar presence/kind, text, boolean-as-text, native numeric text, independent original numeric token, original codec bytes and original source bytes as hex. Native NULL is retained; no COALESCE, token normalization or binary-to-text conversion is introduced. These fixed private aliases are not Weft Column metadata or an adopted public decoder ABI. Payload and structural checks remain independent requirements.

All 66 PostgreSQL crate tests passed (`/private/tmp/weft-row-custody-tests.log`), with fresh final-source row capture. [Native evidence](B-005-row-custody-native.json) covers 16 independently authored vectors (eight each under object/edge captured scalar aliases): Unicode/trailing spaces, empty text/bytes, false/true, uint64, negative-zero lexical custody, deliberately mismatched numeric/token and absent scalar. Actual PostgreSQL 17.9 returns native `0.00` and original `-0.00` separately; mismatch remains observable rather than accepted semantic correspondence. Receipt pins source, capture, harness and Rust log. Synthetic scalar VALUES evidence does not qualify original storage installation, source/codec semantics, tree traversal, logical decoding or host execution/publication. The row-native result bridge remains open B-005 work.

### Bounded private row-custody admission (2026-10-06)

`row_custody::admit` consumes eight optional native text cells in private custody order. It distinguishes physical no-scalar from empty text/bytes, checks selected family and slot separation, accepts only canonical lowercase hex and compares stored codec bytes with the original selected bytes. It retains binary source bytes and native numeric/original-token strings separately. It does not parse source grammar, establish numeric correspondence or interpret physical absence as logical absence. Complete payload/root integrity, native framing, original codec procedures and host authority remain independent requirements.

The bounded API reserves all eight cells and the complete input/copy/hex-decoding byte charge before processing. Failed processing stays charged; exhausted reservations do not partially consume cells. Four new tests cover lexical preservation, malformed/mixed/missing/substituted custody, all 16 recorded PostgreSQL observations and cumulative/refused work. All 70 PostgreSQL crate tests passed (`/private/tmp/weft-row-custody-admission-tests.log`). [Admission receipt](B-005-row-custody-admission.json) pins source, test log and the previously executed native receipt; no new PostgreSQL execution occurred in this admission test. The deliberately mismatched numeric value/token remains retained for later semantic refusal. This private component is not a selected public decoder bridge and does not close row/recursive or B-005 qualification gates.

### Original-property native observation coupling (2026-10-06)

`row_custody::admit_property` couples private scalar cells to an exact admitted property/access cut. It requires both original row home and row location, verifies original value-graph/descriptor custody, derives scalar family and expected codec bytes from the original root, and borrows original presence bytes alongside the admitted physical observation. Input cells cannot select a logical family or codec. Compound roots still require a selected tree decoder. Physical NoScalar remains uninterpreted; this does not grant optional/null semantics or logical decoding.

All 70 PostgreSQL crate tests passed (`/private/tmp/weft-selected-row-custody-tests.log`). The admitted JSONB fixture proves a wrong-home request refuses without consuming native observation budget. [Receipt](B-005-selected-row-custody.json) pins current sources/log and explicitly records that no native execution or positive original-row property integration ran. Raw custody admission retains its previously tested 16 native observations; the original-profile row fixture, selected semantic procedure and public result bridge remain required integration work. No B-005 gate is closed by this coupling.

### Positive original-property/native-home fixture (2026-10-06)

The original Customer/name field fixture now composes a row home from an admitted original join definition and the same original value/presence artifacts. It runs binding admission, original-property/home selection, context/comparator admission, registered row access lowering and property-coupled scalar custody admission. Original codec bytes are retained and compared; text `é  `, binary source bytes `fe00` and original presence bytes survive. Substituting the props property's binding cut refuses. The reused row selector helper is test-only; no production fixture registry or invented native semantic procedure was added.

All 70 PostgreSQL crate tests passed (`/private/tmp/weft-original-row-tests.log`). Fresh final-source capture (`WEFT_ORIGINAL_ROW_CAPTURE`) feeds [two actual PostgreSQL custody queries](B-005-original-row-custody-native.json) against minimal unconstrained object/state/node/scalar tables: no scalar and stored string. Wrong-owner objects are excluded; storage ID `987654321` remains separate from business keys. The harness checks all eight cells including NULL versus empty and exact codec/source hex. Native observations match the independently asserted Rust admission cells; this is component composition, not a complete public execution/decode run. Receipt pins capture, harness, source and log. Registered source/domain artifacts remain synthetic/unqualified, and the selected leaf grammar does not by analogy establish a native-row decoder. Positive original-profile structure is exercised; semantic row decoding, recursive values, complete host obligations and B-005 qualification remain open.

### Direct native-row scalar projection resumed (2026-10-07)

The owner clarified the compiler/storage boundary; Truss runtime adoption is no longer a compiler dependency. `property_projection` now lowers admitted required non-null row scalar roots to native text/boolean columns or original integer/decimal tokens, using UMF-derived Column metadata. It retains owner-wide payload checks over the exact owner/state/node/scalar joins and compares stored codec bytes with the selected original bytes. Rows never fall back to props. Unknown/compound/nullable representations still require their own lowering; full B-005 scope remains open.

All 70 PostgreSQL crate tests passed (`/private/tmp/weft-native-row-projection-tests.log`) and fresh captured original-property string projection passed three actual PostgreSQL 17.9 cases: string result, required absence refusal and wrong-codec refusal. The native harness withholds query execution on violations. This proves direct scalar row compiler behavior over synthetic physical fixtures; source/domain correspondence, complete visibility and execution authority remain explicit host/storage obligations, and no installed production support is claimed. Published Truss checkout was fast-forwarded to d3dcdde; newer draft UMF/DDL source hashes are recorded in the storage-realization pin manifest. Historical adoption blockers in earlier evidence describe prior decisions and are superseded by the 2026-10-07 plan/design correction.

### Four-family native row result carriers (2026-10-07)

The shared row scalar result-carrier selector is now exercised directly by native fixtures. `property_projection` delegates family dispatch to this same selector: string uses text_value, boolean uses canonical native boolean text, integer/decimal uses original numeric_token. Numeric_value is reserved for typed operations and cannot replace lexical result custody.

All 70 PostgreSQL crate tests passed (`/private/tmp/weft-row-carrier-tests.log`), with fresh final-source capture. [Native receipt](B-005-row-result-carriers-native.json) pins sources/capture/harness/log and records 16 actual PostgreSQL 17.9 vectors: eight independent valid values each through object/edge captured scalar aliases. Cases cover empty/Unicode strings, false/true, unsigned 64-bit maximum, signed 64-bit minimum, negative-zero decimal token and exact large decimal scale. Each physical payload predicate returns true and the actual result carrier exactly equals the independent expected string. This qualifies those carrier expressions; full original-property native query evidence remains the separately recorded string fixture, and broader property/type facets, presence/recursive results and public integration remain required work. No production layout/execution support is implied.

### UMF-governed explicit JSON-null presence (2026-10-07)

Corrected the original presence interpreter to follow its pinned `jsonNullMember: present-null-if-authored-nullable` rule. A present JSON null is now observed as present when the original UMF scalar descriptor permits null; nonnullable fields refuse. Absence remains absence under either nullability, and SQL NULL/JSON-null/nonobject roots remain malformed storage roots. Empty strings/collections and false values remain present. No runtime adoption is needed to interpret a rule already fixed by the original schema and UMF descriptor. The existing PropertyAdmission path supplies nullability from its admitted original descriptor.

All 71 PostgreSQL crate tests passed (`/private/tmp/weft-json-null-presence-tests.log`). [Receipt](B-005-json-null-presence.json) pins interpreter/schema/log. New independent cross-nullability cases cover absence, null, empty string/list/map, false and invalid roots. No new database execution occurred. This is pure presence observation, not completed nullable result emission, native-row null-node decoding or SQL-native NULL permission. Those distinctions remain explicit required implementation work.

### Scalar JSONB presence-envelope projection (2026-10-07)

Original-property projection now accepts Scalar descriptor roots whose optional/nullable result uses the existing Value representation. It emits explicit `state: absent`, `state: null` or `state: value` envelopes through the original props location and selected leaf carrier, with scalar numbers still encoded as exact strings. Owner-wide integrity combines root validity with UMF-required/nullable branch legality and original leaf-kind checks. Required non-null scalar carriers are unchanged; compounds and native-row presence still require their corresponding paths. SQL NULL is not used as the public presence carrier.

All 71 PostgreSQL crate tests passed (`/private/tmp/weft-scalar-presence-tests.log`). Fresh SQL capture from an admitted string location/codec, under four explicit required/nullability flag combinations, passed [28 actual PostgreSQL cases](B-005-scalar-presence-native.json). Independent roots cover absent, JSON null, empty text, Unicode/trailing spaces, wrong scalar, nonobject root and SQL NULL root. Only valid branches execute the query; returned envelopes match independently authored expected JSON objects. Receipt pins source, capture, harness and test log. This exercises the envelope primitive's matrix, not full original optional/nullable UMF-model admission or the public V02 backend integration; those remain required broader evidence.

### Native-row scalar presence envelopes (2026-10-07)

Original-property scalar projection now supports its Value representation on row homes. No state row emits absent only for optional fields; a null value_kind emits null only for nullable descriptors and requires no scalar payload plus retained node definition/source custody. Scalar nodes use the original typed payload and codec-byte check. Boolean envelope values remain JSON booleans; exact numeric carriers remain strings. Unknown node kinds, malformed joins and inconsistent null/payload states refuse through independent owner-wide structural/payload checks. Compound roots remain pending recursive lowering.

All 71 PostgreSQL crate tests passed (`/private/tmp/weft-row-presence-tests.log`), with fresh final-source capture. [Native receipt](B-005-row-presence-native.json) records 36 actual PostgreSQL 17.9 cases across four explicit required/nullability flag combinations and nine independent state arrangements: absent, null, empty text, Unicode, null with payload, missing root, wrong kind, missing source bytes and wrong codec. Every valid envelope matches independent expected JSON; any structural/payload violation causes the harness to withhold query execution. Receipt pins source/capture/harness/log. The flag matrix exercises the presence primitive with an admitted original location; full optional/nullable UMF-model compilation, node source/definition semantic correspondence, recursive results and public backend integration remain separately required. This does not claim installed Truss runtime enforcement.

### Bounded recursive value-body traversal (2026-10-07)

Added iterative `value_traversal` over captured finite value topology: scalars, ordered sequences, literal-key maps, structured references and complete record layouts. Runtime values are processed postorder without Rust call-stack recursion. Cyclic metadata does not memoize values: each runtime occurrence and each record member check is charged. Depth, node, member-work and literal output-key byte ceilings refuse before affected traversal/allocation; selected leaf procedures own their separate scalar copy/semantic bounds. Unknown members, repeated stored member names and numeric/container outputs from scalar callbacks refuse. Missing record members invoke the original field-identity/presence-byte callback; no absence/default/null meaning is invented.

`decode_property` verifies original graph/descriptor custody before invoking callbacks and uses the captured layout. The admitted original scalar fixture checks exact codec bytes and preserves Unicode/trailing spaces through this bridge. Three additional tests cover nested exact numeric strings and empty containers, finite values under recursive structured/record metadata, explicit required-member refusal, unknown members and depth/node/key reservation failures. All 74 PostgreSQL crate tests passed (`/private/tmp/weft-recursive-traversal-tests.log`); [receipt](B-005-recursive-traversal.json) pins source/log. No native database run occurred for this pure component. Returned objects preserve stored literal keys as an intermediate body; public logical member/result-envelope realization and native row-tree decoding remain required integration work. This does not close recursive result acceptance.

### One-call original registry compilation (2026-10-07)

`select_definition::compile_with_registry` now owns parameters and performs admitted Record/property preparation plus SELECT assembly in one operation. Successful output includes SQL, Columns, exact ordered parameters and structural/payload prerequisites. Failure cannot return detached preparation or partial parameter state. Native operator lowering remains an explicit trusted backend callback; model content cannot load code. This is a Rust compiler entry point over resolved context and registered originals, not the completed shared Backend trait dispatch or V02 public request path.

The admitted self-join fixture compares complete SQL/parameter/check output with the independently prepared path and verifies incomplete Record registry and refused native operation produce no Compilation. All 74 PostgreSQL crate tests passed (`/private/tmp/weft-integrated-select-tests.log`). Fresh capture from the one-call result passed [three native PostgreSQL cases](B-005-registry-select-native.json): empty, exact duplicate/Unicode bag and complete-owner malformed payload preflight refusal. Receipt pins sources/capture/harness/log. Public plugin integration, full scalar/presence/recursive original-model corpus and V02 remain required B-005 work; this does not establish production support.


### Original definitions through public Backend registration (2026-10-07)

`original_backend::OriginalBackend` connects the admitted original Record/property/comparator registries to the existing Backend trait and Registry compile envelope. Registration binds the exact original binding digest; each compilation rechecks its custody. V01 SELECT compilation retains exact SQL, ordered parameters and result Columns. Complete read-context, owner-wide structural checks and payload checks are emitted as host obligations, including check SQL/parameters and original codec/presence bytes. Hosts remain responsible for enforcing these obligations before query execution/publication.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-original-backend-tests.log`). The admitted original self-join fixture now exercises actual public Registry dispatch, compares emitted SQL/parameters with the one-call compilation, checks retained preflight obligations, and confirms candidate-disabled refusal. Its native operator callback is a trusted fixture procedure for field/equality expressions. No new database or Python/browser run occurred. This candidate V01 bridge does not qualify V02, recursive results, generic native operators or production Truss support; those remain separate B-005 work.


### Original-definition V02 field SELECT assembly (2026-10-07)

The one-call original registry compiler now assembles V02 direct field projections using admitted scan/property sources and the existing result ABI. It retains owner discriminator filters, structural checks, payload checks and exact parameters. Field results reach the selected property projection before the computed-scalar guard, allowing scalar-root presence envelopes when admitted descriptors require Value representation. Compound roots still require their recursive result bridge. V02 joins, filters, grouping, aggregates, ordering, limits and computed outputs explicitly refuse until their original-definition lowerings exist; no stage is silently dropped. Public OriginalBackend registration remains V01-only pending broader V02 coverage.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-application-select-tests.log`). The admitted original required-string fixture exercises V02 one-call compilation and confirms an unimplemented limit refuses. Fresh captured SQL passed [three actual PostgreSQL cases](B-005-application-select-native.json): empty bag, exact duplicates/Unicode/empty string, and malformed owned payload preflight refusal. Single-column empty CSV rows are decoded as empty strings by this text-only fixture harness. Receipt pins source, capture, harness and test log. This does not yet establish optional/nullable original-model compilation, recursive results, complete V02 lowering or Python/browser dispatch.


### Fieldless original-definition V02 COUNT assembly (2026-10-07)

The original registry SELECT assembler now lowers an aggregate projection containing only COUNT(*) outputs over its admitted single owner source. COUNT returns PostgreSQL bigint as text with the existing exact-integer result decoder. Its resolved logical type must be a nonnullable integer with the COUNT resolver's empty facets. Aggregate/projection stage mismatches refuse; mixed field/aggregate outputs still require grouping implementation. No property registration, property payload access or property structural check is invented for fieldless counting. Other V02 relational stages retain their explicit refusals.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-application-count-tests.log`), including original Record-only registration, one exact owner parameter, no payload checks, and aggregate-stage refusal. Fresh capture passed [three native PostgreSQL cases](B-005-application-count-native.json) on a table with only id/type_id columns: empty owner bag, two owned rows, and five duplicate-ID rows. Each includes an unrelated owner excluded by the emitted discriminator. The native result is compared with independently authored counts and exact-integer metadata. Receipt pins source/capture/harness/log. Public OriginalBackend remains V01-only; full grouped aggregates, V02 backend registration, original optional/nullable/recursive corpus and Python/browser integration remain B-005 work.


### Original-definition string grouping with COUNT (2026-10-07)

V02 SELECT assembly now admits COUNT with original nonnullable string group fields. Group expressions are lowered through the trusted backend native callback after selected comparator operation/type/original-property admission. Projected group fields use the same lowered expression as GROUP BY, retaining the exact text result carrier. Missing comparators, repeated groups, aggregate-stage mismatches and projected nongroup fields refuse. Parameters commit only after complete assembly. Numeric/boolean group carrier correspondence, SUM, predicates, joins, ordering and limits remain pending; this bridge does not choose their semantics by extrapolation.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-application-group-tests.log`). The original admitted string fixture exercises grouped one-call lowering and missing-comparator refusal before the callback. Fresh capture passed [three actual PostgreSQL cases](B-005-application-group-native.json): empty groups; a bag with duplicates, case distinctions, empty text, trailing spaces and composed/decomposed Unicode; and malformed owned payload preflight refusal. The selected fixture callback explicitly uses C collation. Independent Python string grouping supplies expected names/counts without relying on SQL expressions. Receipt pins source/capture/harness/log. This is original-definition component evidence; public V02 registration, the complete semantic corpus, recursive results and embeddings still need implementation and verification before B-005 closes.


### Original-definition V02 ordering and LIMIT (2026-10-07)

V02 SELECT assembly now lowers ascending order fields through the trusted native callback after original comparator ordering admission. Aggregate ordering must reference admitted group fields and reuses their lowered expressions. LIMIT preserves the bounded resolved integer (1–1000); out-of-range values refuse. Ordering/limits apply after projection/grouping while complete-owner structural and payload prerequisites retain their independent unfiltered/unlimited SQL. Parameters still commit atomically. This does not provide cursor predicates or qualify authored page-key/profile admission through the public backend.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-application-order-tests.log`), including admitted ordering plus LIMIT 3 and invalid LIMIT refusal. Fresh capture passed [three actual PostgreSQL cases](B-005-application-order-native.json): empty page, exact ordered three-row page from duplicate/case/Unicode/empty-string rows, and complete-owner malformed payload preflight refusal. Expected row order uses independently computed UTF-8 byte ordering for the fixture's explicit C comparator; comparison is ordered rather than bag-based. Receipt pins source/capture/harness/log. Public V02 registration, cursor/relationship predicates, numeric group correspondence, SUM, recursive result lowering and embedding verification remain required B-005 work.


### Original-definition V02 equality filters (2026-10-07)

The original registry SELECT assembler now lowers V02 field/literal and field/field equality filters through the existing typed Expression traversal and trusted native callback. Original comparator equality admission occurs before native SQL generation. Owner discriminator filters are retained and user filters apply before grouping, ordering and limits. Literal conversion is callback-owned and emits typed parameter slots; source text is not interpolated into SQL. Named parameters explicitly refuse until their origin-preserving bridge exists. Cursor/relationship predicates and joins still require corresponding original-definition lowerings.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-application-filter-tests.log`). The admitted string fixture verifies the literal occupies the third parameter after owner/member selection. Fresh capture passed [three actual PostgreSQL cases](B-005-application-filter-native.json): empty filtered result, exact duplicate matching strings, and malformed owned payload detected by independent full-owner preflight even though it cannot match the filter. Receipt pins source/capture/harness/log. The native callback is a trusted fixture using C text comparison, not qualified generic operator coverage. Public V02 registration, complete original-model scalar/presence/recursive tests and embeddings remain required before B-005 completion.


### Original-definition V02 equality joins (2026-10-07)

The original registry SELECT assembler now composes inner joins in plan order over independently admitted owner sources. Conditions use the same selected equality bridge as filters. Each right owner retains its discriminator filter and its complete-owner structural/payload prerequisites. Repeated occurrences, empty conditions, missing prepared sources and references to later scans refuse. SQL uses simple table factors directly and parentheses for right sources with row-home join trees; the row-home multi-scan composition still needs independent native coverage. Named parameters and non-equality join conditions retain explicit refusals.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-application-join-tests.log`). The original admitted string self-join verifies separate occurrences, four ordered owner/member parameters, two structural and two payload prerequisites. Fresh capture passed [three PostgreSQL cases](B-005-application-join-native.json): empty bag, exact duplicate/Unicode self-join bag, and malformed owned payload preflight refusal. Native execution caught and corrected invalid parentheses around a simple aliased table before final capture. Receipt pins final source/capture/harness/log. This is original-definition V02 component evidence with a trusted C-text fixture callback; public registration, row-home/multi-owner join corpus, other predicates, SUM, recursive results and embeddings remain B-005 work.


### Original-definition native-row V02 self-join (2026-10-07)

The original admitted row-home fixture now re-admits its Record source against the same row binding cut and compiles a V02 self-join through the one-call registry compiler. Its trusted fixture operator reads the typed text scalar and uses C equality. Both occurrences retain independent native state/node/scalar access trees, six exact parameters, original codec custody and owner-wide preflight checks. This supplies native evidence for the right joined-table factor introduced by the preceding V02 join implementation.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-application-row-join-tests.log`). Fresh capture passed [four PostgreSQL cases](B-005-application-row-join-native.json) over synthetic typed-row tables with no props column: empty bag; exact duplicate/Unicode/empty-text self-join bag; wrong original codec bytes; and missing required scalar state. Invalid storage causes the harness to withhold the query after complete-owner checks. Receipt pins source/capture/harness/log. These are compiler fixtures for the documented storage realization, not installed Truss runtime or generic codec qualification. Multi-owner/edge joins, the full original-model corpus, public V02 dispatch, recursive results and embeddings remain separate B-005 acceptance work.


### Public Registry V02 original-definition dispatch (2026-10-07)

OriginalBackend now accepts the V02 dialect/IR pair through the existing public Registry compile envelope. V02 capability declarations are limited to implemented scalar relational, COUNT/string grouping, ordering/LIMIT and scalar-root presence stages. Named parameters, cursor comparison, page-key, recursive/entity/relationship projections and SUM have no V02 declaration. Lowering still validates selected originals and refuses unsupported plan shape atomically; candidate declarations do not assert all model/type combinations are supported.

All 74 PostgreSQL crate tests passed (`/private/tmp/weft-public-application-tests.log`). The public V02 self-join verifies exact parameter/result metadata against the original one-call path, retains execution obligations, and refuses a required named-parameter capability absent from its V02 declaration. Fresh SQL captured from the public Emission passed [three actual PostgreSQL cases](B-005-public-application-native.json): empty, duplicate/Unicode self-join bag and complete-owner malformed payload preflight refusal. The fixture native callback remains trusted fixture code; native comparison runs on the recorded C-locale PostgreSQL instance. Receipt pins source/capture/harness/log. Complete V02/recursive lowering, full original-model semantic corpus and Python/browser registry packaging remain required B-005 work; this supersedes earlier V01-only registration notes without claiming full slice or production completion.


### Original-definition V02 SUM bridge preparation (2026-10-07)

V02 SELECT assembly now routes SUM through the existing typed Expression traversal and trusted native callback after original comparator SUM admission. Result validation preserves the resolver's exact numeric family, decimal scale and ungrouped empty-input nullability. Unsupported argument families and altered result types refuse. This extends the internal one-call path only: public V02 SUM capability remains undeclared until admitted numeric original-model/native execution evidence exists.

All 75 PostgreSQL crate tests passed (`/private/tmp/weft-application-sum-tests.log`). The new result-contract test covers uint64 and decimal(28,9), grouped/ungrouped nullability, altered scale/facets and string refusal. No native database or embedding run occurred for this checkpoint. Positive admitted numeric compilation, exact native aggregate results/overflow behavior and public SUM dispatch remain required acceptance work; this evidence does not qualify completed aggregate support.


### Selected exact numeric SUM procedure (2026-10-07)

Original native comparator definitions now expose `sum_sql` for registered SUM operations and closed exact native numeric types. Altering native type away from original comparator bytes refuses; floating-point strategies cannot supply SUM. Carrier SQL is trusted physical-plan input. Source/domain integrity and transport checks remain prerequisites; casting does not itself prove them.

All 76 PostgreSQL crate tests passed (`/private/tmp/weft-numeric-sum-tests.log`). Fresh comparator SQL capture passed [four native PostgreSQL cases](B-005-numeric-sum-native.json): uint64 and decimal empty-input NULL, two uint64 maxima summed beyond uint64 range, and exact decimal scale-nine cancellation. Expected text results are authored independently. This qualifies the selected SQL primitive over synthetic text inputs only. Complete numeric original-property admission/SELECT compilation, source-domain refusal, public V02 SUM capability and embedding coverage remain pending.


### Comparator-owned SUM in original registry compilation (2026-10-07)

The one-call compiler now intercepts typed field SUM and resolves its comparator by exact prepared scan occurrence plus original owning property. It rechecks property binding custody and argument type before invoking the comparator-owned numeric SUM procedure. Missing ownership/selection, computed arguments without their own bridge and wrong operand arity refuse. Trusted callbacks continue to lower scalar operands and other operators; they no longer choose field SUM SQL in this entry point. Native strategy as well as native type must match the comparator's original bytes.

All 76 PostgreSQL crate tests passed (`/private/tmp/weft-owned-sum-tests.log`), including added strategy-substitution refusal. This connects the separately native-tested numeric SQL primitive to compilation, but the current full original-property fixture remains string-only. No new native or embedding execution occurred. Full admitted numeric-property SUM compilation and execution remains unproven; public V02 SUM capability remains undeclared pending that evidence.


### Original decimal-property SUM compilation and execution (2026-10-07)

Added a positive decimal(28,2) fixture from the existing original UMF Orders.total definition. It admits the original value graph, numeric leaf/presence definitions, props home, independent Record source and SUM comparator against one binding cut. Both V01 and V02 compile through the one-call registry path with identical SQL and result Columns. The fixture callback supplies only field access; a callback SUM invocation panics, proving selected comparator ownership in this path. Its synthetic numeric evidence/profile artifacts are fixture registrations and do not assert Truss runtime adoption.

All 77 PostgreSQL crate tests passed (`/private/tmp/weft-original-numeric-tests.log`). Fresh capture passed [three PostgreSQL cases](B-005-original-numeric-native.json): empty-input SQL NULL; exact scale-two total near 10^19 with cancellation; and wrong scalar kind rejected by complete-owner preflight. An unrelated malformed owner is excluded by the admitted discriminator. Receipt pins final source/capture/harness/log. This closes the positive original decimal-property SUM integration evidence gap for this props fixture. Complete numeric source grammar/facet refusal, unsigned/native-row aggregate corpus, public V02 SUM qualification and embedding integration remain required B-005 work.


### Original SUM native-domain prerequisites (2026-10-07)

The one-call compiler now appends independent complete-owner numeric domain checks for selected SUM reads. Comparator-owned SQL checks native numeric input validity before guarded casts, rejects nonfinite values, and enforces original decimal precision/scale or integer integrality/range without rounding. Checks remain separate from user filters, grouping, order and limits; hosts must enforce them before executing/publishing results. This is native-domain enforcement, not original source-token grammar admission, which remains a separate codec obligation. Row homes use their retained native numeric carrier; complete numeric token correspondence remains separate required evidence.

All 77 PostgreSQL crate tests passed (`/private/tmp/weft-sum-domain-tests.log`). Fresh original decimal SUM capture passed [seven native cases](B-005-sum-domain-native.json): empty, exact large scale-two total, wrong storage kind, excessive scale, excessive precision, NaN and invalid native text. For each numeric-domain negative, all preceding physical checks pass and only the added numeric check fails; the harness withholds aggregation. Receipt pins source/capture/harness/log. Integer/native-row domain matrices, original token grammar/correspondence, public SUM qualification and embeddings remain required B-005 work.


### Original uint64 SUM integration and shared numeric fixture (2026-10-07)

The complete numeric-property fixture now shares one constructor for original record/member, scalar family and selected comparator. Added Customer.id uint64 admission without changing its original UMF bytes or facets. Both V01 and V02 preserve identical comparator-owned SUM SQL and result Columns. Native SUM widens the result to exact arbitrary-precision integer text; operand width still constrains each stored value independently.

All 78 PostgreSQL crate tests passed (`/private/tmp/weft-original-integer-tests.log`). Fresh capture passed [eight uint64 PostgreSQL cases](B-005-original-integer-native.json): empty NULL, two maxima summed beyond uint64, wrong kind, fraction, negative, operand overflow, Infinity and invalid native text. For each numeric negative, physical checks pass and the numeric preflight fails before aggregation. [Seven refreshed decimal cases](B-005-original-decimal-refresh-native.json) also pass after fixture extraction. Receipts pin source/capture/harness/log. These are synthetic original-admitted props fixtures; numeric row-home aggregates, source-token correspondence, public SUM qualification and embeddings remain required B-005 work.


### Public V02 SUM dispatch with original numeric definitions (2026-10-07)

OriginalBackend now declares candidate V02 SUM and dispatches it through its exact original-property/comparator path. Decimal(28,2) and uint64 fixtures register owned Record/property/comparator definitions, verify public SQL/result metadata against the internal compilation and retain all three physical/numeric owner prerequisites in Emission obligations. Candidate-disabled requests refuse. This supersedes earlier undeclared-SUM notes for this scoped candidate path; broader selected domains still require their own meaning and evidence.

All 78 PostgreSQL crate tests passed (`/private/tmp/weft-public-sum-tests.log`). SQL captured directly from public Emission passed [seven decimal](B-005-public-decimal-native.json) and [eight uint64](B-005-public-integer-native.json) PostgreSQL cases, including empty-input nullability, exact large totals and independent physical/native-domain refusals. Harnesses locate the numeric prerequisite by its SQL meaning rather than assume obligation ordering. Receipts pin source/capture/harness/log. Numeric row-home correspondence, complete recursive/model corpus and Python/browser original registry packaging remain required B-005 work. No production compatibility is inferred from candidate registration.


### Origin-preserving named equality parameter bridge (2026-10-07)

V02 equality lowering now preserves named parameter identity and span while reusing selected literal conversion. The callback must emit exactly one parameter slot retaining the exact supplied value and logical type; inline-only conversion, multiple slots or altered values/types refuse. The slot receives the existing parameter/span origin shape. Expression-local staging and the one-call compiler keep failures atomic. This bridge applies to equality filters and join conditions; cursor tuples remain separate pending work.

All 78 PostgreSQL crate tests passed (`/private/tmp/weft-named-origin-tests.log`). The original string fixture compares named versus literal SQL, checks retained name metadata and verifies inline SQL without a parameter slot refuses. No new database or embedding run occurred. Public V02 named-parameter capability remains undeclared pending public request/transport evidence; previous internal named-equality refusal notes are superseded only for this scoped bridge.


### Public Registry named equality parameters (2026-10-07)

OriginalBackend now declares candidate V02 named parameters for its implemented equality bridge. The original string fixture compiles a named filter through public Registry, preserving exact value, type and parameter/span origin in the existing Emission ABI. Unsupported relationship capability requests still refuse. Cursor predicates remain separately unsupported; this declaration does not supply their lowering.

All 78 PostgreSQL crate tests passed (`/private/tmp/weft-public-named-tests.log`). Fresh public Emission passed [three actual PostgreSQL cases](B-005-public-named-native.json): empty result, exact duplicate matching rows and complete-owner malformed storage preflight refusal. The native harness binds the recorded three-slot envelope and asserts retained named origin. Receipt pins source/capture/harness/log. This supersedes prior undeclared-named notes for the scoped Rust Registry path; public frontend/embedding packaging, cursor/relationship operations and complete recursive/model coverage remain required B-005 work.


### Original-definition cursor tuple bridge preparation (2026-10-07)

V02 filter lowering now renders lexicographic greater-than tuples using ordered selected field/value carriers and PostgreSQL ROW comparison. Components reuse the origin-preserving conversion bridge and selected comparator admission; matching nonnullable logical types are required. Empty, mismatched and greater-than-32-component tuples refuse under this implementation bound. Parameters commit only after complete tuple lowering. Named values keep their source identity/span. This implements the comparison primitive, not authored page-key correspondence or public paging qualification.

All 78 PostgreSQL crate tests passed (`/private/tmp/weft-cursor-bridge-tests.log`). The admitted original string fixture exercises a single named cursor component and retained parameter metadata. Fresh capture passed [three PostgreSQL cases](B-005-cursor-bridge-native.json): empty, exact rows above the selected string and malformed full-owner preflight refusal. Receipt pins source/capture/harness/log. Composite component ordering, boundary/refusal matrices, native numeric cursor cases, original page-key admission and public cursor capability remain required B-005 work; no complete cursor support is claimed.


### Composite cursor ordering and bounds (2026-10-07)

Added an admitted original two-scan string cursor fixture with independently named components. Its equality join produces a full owner bag so both greater-first-component and equal-prefix/greater-second-component branches are observable. Component parameter origins remain in tuple order after four owner/member slots. New bound tests prove empty, mismatched and 33-component tuples refuse before callbacks and return no parameter state.

All 79 PostgreSQL crate tests passed (`/private/tmp/weft-composite-cursor-tests.log`). Fresh capture passed [three native PostgreSQL cases](B-005-composite-cursor-native.json): empty tuple-filtered bag, duplicate/Unicode bag compared independently with Python tuple greater-than, and malformed complete-owner preflight refusal. Receipt pins source/capture/harness/log. This supplies composite comparison primitive evidence, not authored unique page-key correspondence, numeric cursors or public paging qualification; those remain required B-005 work.


### Original Record page-key correspondence (2026-10-07)

Record admission now retains its original pinned Record. `verify_key_mapping` rechecks model/binding cuts, resolves the original authored key, compares complete resolved identity/field order/types, matches exactly one physical owner/key mapping, verifies accepted key bytes against the original Record and checks ordered property IDs. V02 one-call compilation invokes this check when a page key is selected. Encoding/comparison procedure admission and uniqueness enforcement remain separate; correspondence alone does not qualify paging.

All 80 PostgreSQL crate tests passed (`/private/tmp/weft-key-correspondence-tests.log`). The application fixture's authored Record keys pass; substituted field identity and nullability/type fail. No native database or embedding run occurred. Full original-admitted page/cursor integration, selected physical key procedure meanings and public paging qualification remain required B-005 work.


### Complete page-key order and cursor coupling (2026-10-07)

Original one-call V02 compilation now checks page-key order before preparing physical accesses. Ordering must match the complete original source key's fields/types in order and remain on the source scan, with a bounded LIMIT. Cursor predicates must compare that same complete ordered key against matching typed literal/named values; field-valued or partial cursor tuples refuse. Original Record/physical mapping correspondence still follows independently. This prevents valid key metadata from accompanying a different paging operation.

All 81 PostgreSQL crate tests passed (`/private/tmp/weft-page-order-tests.log`). A real application frontend-resolved entity page passes; removed order, substituted scan, missing LIMIT and incomplete cursor fail. No native database or embedding run occurred. Full original-admitted native page execution, physical key comparison/encoding procedure qualification and public paging capability remain required B-005 work.


### Original uint64 entity-page execution (2026-10-07)

The shared numeric original-property fixture now uses the application model, retaining its authored keys. Its uint64 variant resolves a real entity-page request through the application frontend and compiles it through original Record/property/comparator admission, complete key/physical property-order correspondence, numeric ordering and LIMIT 2. Fixture comparator registrations include key/equality/ordering meanings alongside SUM; the trusted page field procedure explicitly casts to numeric for ordering while projection preserves exact original text.

All 81 PostgreSQL crate tests passed (`/private/tmp/weft-original-page-tests.log`). Fresh capture passed [three PostgreSQL cases](B-005-original-page-native.json): empty page; values 10, 2, uint64 maximum and 3 yielding exactly 2 then 3; and malformed owned storage detected by complete-owner preflight before limited query execution. Storage object IDs intentionally differ from business-key values. Receipt pins source/capture/harness/log. This establishes native ordering/limit/key-correspondence execution on valid unique synthetic values, not storage uniqueness enforcement, source-domain qualification for all ordering operands, physical key encoding/comparison procedures or public paging capability. Those remain B-005 acceptance work.


### Complete-owner page-key uniqueness and native domains (2026-10-07)

Original page compilation now emits a full-owner duplicate-key grouping prerequisite using selected field lowering independently of filters/order/LIMIT. Numeric key operands also receive finite/integrality/range/precision/scale prerequisites. Numeric domain SQL no longer requires a SUM operation registration; it uses the captured original closed numeric strategy. Uniqueness expressions guard numeric casts with domain checks so malformed values do not depend on prerequisite execution order. Preflight field rendering cannot add/change prepared parameters.

All 81 PostgreSQL crate tests passed (`/private/tmp/weft-page-uniqueness-tests.log`). Fresh page capture passed [five PostgreSQL cases](B-005-page-uniqueness-native.json): empty, exact numeric order, malformed storage hidden by LIMIT, duplicate logical keys and negative uint64 key. Each invalid case withholds the page after complete-owner prerequisites. Receipt pins source/capture/harness/log. This establishes native props uniqueness/domain observation under the fixture comparator; physical key encoding meanings, numeric row-home correspondence, public paging and complete recursive/embedding coverage remain B-005 work.


### Public original-definition entity paging (2026-10-07)

OriginalBackend now advertises candidate V02 unique stable keys and lexicographic comparison. The uint64 application fixture resolves an entity page through the frontend and public Registry, retaining original UMF key correspondence, numeric ordering, exact text projection and four complete-owner prerequisites. The fixture native field procedure casts numeric carriers for ordering; public SUM parameter/result contracts remain unchanged.

All 81 PostgreSQL crate tests passed (`/private/tmp/weft-public-page-tests.log`). Fresh public emission passed [five PostgreSQL page cases](B-005-public-page-native.json): empty, exact numeric order, malformed storage hidden by LIMIT, duplicate logical keys and invalid uint64 keys. Fresh SUM emissions also passed [seven decimal cases](B-005-public-decimal-page-refresh-native.json) and [eight integer cases](B-005-public-integer-page-refresh-native.json). These qualify the synthetic Rust Registry props page path; public numeric cursor execution, row-home paging, recursive values and embedding coverage remain B-005 work. Unused physical key encodings do not gate this property-comparator query path.


### Public original-definition numeric cursor paging (2026-10-07)

The original uint64 fixture now resolves a named-cursor entity page through the application frontend and public Registry. Its trusted native procedure binds numeric literals as exact text cast to numeric; the cursor bridge retains named origin and original type. Original authored key/order correspondence and all four complete-owner prerequisites remain coupled to the page.

All 81 PostgreSQL crate tests passed (`/private/tmp/weft-public-cursor-tests.log`). Fresh emission passed [eight PostgreSQL cases](B-005-public-cursor-native.json): empty, numeric continuation, adjacent integers beyond JavaScript precision, equality/below-cursor exclusion, uint64 maximum, malformed storage hidden by LIMIT, duplicate keys below the cursor and invalid uint64 keys below the cursor. Python integer comparison independently computes expected ordered results. Corrupted owners withhold execution after prerequisites; foreign owners remain outside the read cut. Receipt pins source, harness, emission and test log. This qualifies the synthetic Rust Registry props cursor path; composite authored pages, row-home numeric paging, recursive values and embedding coverage remain B-005 work.


### Recursive pending-work reservation (2026-10-07)

Original-layout traversal now charges root and child occurrences before queueing work. Sequence/map/record children and structured references share the same node budget, including metadata cycles. Previously nested wide containers could queue uncharged children while older siblings waited. Failed reservations retain the remaining counter and return no decoded value.

All 82 PostgreSQL crate tests passed (`/private/tmp/weft-recursive-reservation-tests.log`). A cyclic sequence topology exercises early refusal at six nodes and exact success at seven, including empty nested sequences when the remaining budget reaches zero. Existing recursive presence, literal-key and exact scalar tests still pass. [Receipt](B-005-recursive-reservation.json) pins source and log. This is traversal resource evidence; recursive SQL emission, logical structured-member envelopes, native row trees and public embedding integration remain required B-005 work.


### Original-slot logical record assembly (2026-10-07)

Recursive traversal now offers a record assembly hook after child decoding. Every original member slot is supplied in declared order with its identity, stored name, original presence bytes and optional decoded value. Missing members remain explicit after selected absence validation, allowing logical member names and presence envelopes to be assembled without inferring meaning from storage keys. A non-object assembled record refuses. The existing storage-body decoder retains its prior behavior; an original-property entry point rechecks graph/descriptor custody before all callbacks.

All 83 PostgreSQL crate tests passed (`/private/tmp/weft-record-assembly-tests.log`). A test proves original slot order and custody, logical name replacement, explicit absent envelope, exact Unicode/trailing-space retention and non-object refusal. [Receipt](B-005-record-assembly.json) pins source and log. This supplies the assembly interface, not a qualified automatic UMF member decoder; original compound property fixtures, selected nullable-container semantics, recursive SQL and embedding execution remain B-005 work.


### Descriptor-selected logical member names (2026-10-07)

`decode_logical_property` now couples original-property graph/descriptor custody to record assembly. Each storage slot resolves its exact UMF field descriptor and authored Record member name, retaining original presence bytes and explicit missing/present value for a selected member-envelope procedure. Duplicate logical names, missing membership and conflicting names for one identity refuse before envelope callbacks. Logical output-name bytes are charged independently of storage-name traversal bytes. Record assembly callbacks now receive the shared traversal budget.

All 84 PostgreSQL crate tests passed (`/private/tmp/weft-logical-members-tests.log`). Tests prove storage-to-authored name mapping, original descriptor availability/presence bytes, exact Unicode text, missing/present envelopes, duplicate/ambiguous name refusal and name-budget refusal before callbacks. [Receipt](B-005-logical-members.json) pins source/log. The selected member procedure still owns null/availability semantics; this bridge does not infer them. Full original compound fixtures, recursive SQL/native row trees and Python/WASM integration remain B-005 work.


### Captured original member presence and default envelopes (2026-10-07)

Value admission now retains independent copies of selected original record-member presence definitions. The default logical-property decoder rechecks their graph correspondence and exact bytes before member procedures. Required members cannot be absent; absent-allowed members emit explicit absent envelopes. Scalar null requires authored nullable permission and emits a null envelope; optional/nullable present values emit value envelopes. Required nonnullable values remain bare logical bodies. Unspecified availability and explicit compound null without selected model meaning refuse. Original leaf decoding remains a separately selected codec procedure.

All 85 PostgreSQL crate tests passed (`/private/tmp/weft-member-presence-tests.log`). New cases cover availability/nullability combinations, empty string versus absence/null, empty required sequence, unknown availability and unsupported compound null. The admitted original scalar property fixture also executes the new entry point with exact original codec bytes and Unicode/trailing spaces. [Receipt](B-005-member-presence.json) pins all changed sources and log. Full admitted compound fixtures, recursive SQL/native row trees and Python/WASM execution remain B-005 work; no new native database qualification occurred.


### Original-admitted compound logical decoding (2026-10-07)

Added sequence (`Customer.tags`) and structured (`Customer.address`) fixtures from the exact original application UMF module. Their finite value graphs retain each original authored definition, original leaf codec bytes and independently captured record-member presence definitions. Value admission rechecks original model sources/topology before the default logical decoder traverses the body. Structured storage slots deliberately differ from authored member names. The test's physical home is a placeholder and is not used or qualified; this evidence is original value admission/decoding only.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-compound-original-tests.log`). Cases preserve exact Unicode/trailing spaces, literal string punctuation, empty sequences, empty required strings and optional present/absent members; numeric JSON string leaves, missing required members, nonnullable null, unknown or substituted storage names refuse. Removing captured original record presence refuses before leaf callbacks. Each leaf callback asserts its exact admitted codec bytes/family/representation. [Receipt](B-005-original-compound-decoding.json) pins source, decoder and test log. Recursive SQL/native row trees, map/nested exact numeric fixtures and Python/WASM integration remain B-005 work.


### Original-topology recursive PostgreSQL observation (2026-10-07)

Added a recursive JSONB physical observation primitive over the admitted finite graph. It rechecks original descriptor/member-presence custody, packs literal stored slots and topology into a bound parameter, and obtains scalar guards from each captured original leaf codec. PostgreSQL walks sequence/map/structured/record occurrences, checks required/absent-allowed availability, authored scalar nullability, container kinds and unknown record slots, and reports depth/count violations. This primitive returns per-value integrity; complete-owner query orchestration and logical projection remain separate integration work. Depth is capped at 128 and observations reject more than 100,000 occurrences after traversal; the count guard is not a claim of bounded database work allocation.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-recursive-observation-tests.log`). Fresh original sequence/structured observations passed [16 native PostgreSQL cases](B-005-original-compound-observation-native.json), covering exact/empty sequences, optional present/absent members, required omission, wrong kinds, nulls and unknown/substituted slots. Native execution exposed untyped SQL NULL inference; the recursive seed now explicitly casts to JSONB and the complete matrix passed after correction. Receipt pins sources, captures, harness and test log. No public recursive projection, row-tree or embedding qualification is claimed; those remain B-005 work.


### Complete-owner original compound props observation (2026-10-07)

The sequence/structured fixtures now admit their physical props homes from original binding definitions and pinned inventory/relation/column correspondence, replacing the earlier unused placeholder home. A complete-owner observation helper lowers owner/member parameters from that admitted home and counts root or recursive payload violations independently of any query filter/order/limit. Parameter changes commit only after complete original graph/codec observation construction. Recursive native row homes still refuse explicitly.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-recursive-owner-tests.log`). Fresh helper emissions passed [12 PostgreSQL cases](B-005-original-compound-owner-native.json): empty owner, exact compound values, malformed root, malformed payload alongside valid rows, original required/optional absence and explicit null, for both sequence and structured homes. Every case includes a malformed foreign-owner row excluded by the admitted discriminator. Receipt pins source/captures/harness/log. This supersedes the placeholder-home limitation for the current compound fixtures, but does not qualify public recursive projection or native row-tree decoding. Select-assembly integration, recursive logical result SQL and embedding evidence remain B-005 work.


### Recursive logical JSONB body SQL (2026-10-07)

The original recursive observation now shares a path-aware walk with logical body construction. Bottom-up Record patches replace stored slot names with authored UMF names after descendant values have been processed. Optional members retain absent/value envelopes, authored nullable scalar members retain null/value envelopes, and required nonnullable bodies stay bare. Sequence indexes and literal map/record keys are preserved as path components; structured references retain the same physical path. Duplicate authored Record names refuse before parameter publication. The returned body is publishable only after original physical/owner prerequisites pass.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-recursive-body-tests.log`). [Fresh native receipt](B-005-original-compound-body-native.json) covers 16 observation cases and independently checks logical JSON bodies for the four valid present sequence/structured cases. Empty arrays, Unicode/trailing spaces, required empty strings and optional absent/present envelopes match Python expectations. [Twelve complete-owner cases](B-005-original-compound-owner-native.json) were refreshed against the same emitted walk. Receipt pins sources/captures/harness/log. Public select assembly, nested map/exact numeric and recursive row-tree cases, and embedding integration remain B-005 work; no complete recursive backend qualification is claimed.


### Parameter-aware recursive props projection (2026-10-07)

Both original SELECT assembly paths now use parameter-aware payload observations and property projection. Compound props use original recursive logical-body SQL inside the public Value presence envelope, with independent full-owner root/recursive integrity checks. Scalar projection stays on its existing original codec path. Graph parameter publication is staged across complete projection/observation construction; native recursive row projection still refuses explicitly.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-recursive-projection-tests.log`). The original sequence/structured fixtures resolve real frontend field projections, lower exact occurrence-qualified physical accesses and emit parameter-aware projection SQL/result metadata. Fresh captures passed [12 PostgreSQL cases](B-005-original-compound-projection-native.json): empty owner, exact values, empty sequence/required strings, required/optional absence, explicit null and malformed values alongside valid rows. Valid text cells decode to exact public absent/value envelopes with authored member names; invalid owners withhold projection execution. A malformed foreign-owner row remains excluded in every case. Receipt pins source/captures/harness/log. This qualifies the original access/projection path; full one-call/public Registry compound fixtures, nested numeric/map and recursive row-tree execution and embedding integration remain B-005 work.


### One-call and public Registry compound SELECT (2026-10-07)

Original compound fixtures now admit their independent source Record and compile frontend-resolved sequence/structured selections through both one-call original SELECT assembly and public Registry. The trusted scalar operator callback must never run for compound direct projection. Public SQL and all four parameter slots exactly match one-call output: owner, property member, full-owner graph observation and projection graph. OriginalBackend declares candidate V02 sequence/structured capabilities for this props path; recursive native row homes continue to refuse.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-public-compound-tests.log`). Fresh [one-call emissions](B-005-original-compound-select-native.json) and [public Registry emissions](B-005-public-compound-select-native.json) each passed 12 native PostgreSQL cases. Complete-owner prerequisites precede execution; valid final cells match exact logical envelopes/member names, invalid owners withhold the SELECT, and foreign owners stay excluded. Receipts pin source/captures/harness/log. This supersedes the prior missing public sequence/structured props fixture evidence. Public map/nested numeric, recursive row-tree, relationship operations and Python/WASM integration remain B-005 work; no broad or production qualification is claimed.


### Original-admitted public map SELECT (2026-10-07)

Extended the compound fixture to the existing `map-page-props` application's exact original UMF map definition. The same original graph/leaf/presence/home/Record admissions now produce logical map decoding, one-call SELECT and identical public Registry SQL/parameters. OriginalBackend declares candidate V02 map capability for this recursive props path. Literal map keys remain data, including empty strings, punctuation and Unicode.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-original-map-tests.log`). Fresh [one-call map emissions](B-005-original-map-select-native.json) and [public map emissions](B-005-public-map-select-native.json) each passed seven native PostgreSQL cases: empty owner, exact literal keys/Unicode/trailing spaces, empty map, required absence, explicit null, malformed leaf alongside a valid map and array-kind substitution. Valid cells match independently specified logical map envelopes; invalid owners withhold SELECT execution. Receipts pin sources/capture/harness/log. This supplies the previously missing public map props fixture; nested exact numeric/recursive compound coverage, native row trees, relationships and embedding integration remain B-005 work.


### Original nested numeric sequence and cyclic structured SELECT (2026-10-07)

Extended compound admissions to the existing nested-sequence and cyclic-structured original UMF fixtures. Nested sequence leaves retain the original uint64 codec/adoption artifact and exact source text; the selected Rust fixture decoder checks uint64 domain before returning a body. Cyclic metadata stays finite while every runtime Record occurrence is transformed independently, including absent optional recursive members. Both compile through the one-call and public Registry paths. The generalized fixture now constructs graph scalar families from exact original descriptors; its prior string-only assumption was removed.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-nested-original-tests.log`). Fresh [one-call nested emissions](B-005-original-nested-select-native.json) and [public nested emissions](B-005-public-nested-select-native.json) each passed 14 PostgreSQL cases: exact nested uint64 values beyond JavaScript precision, uint64 maximum, empty nested arrays, two recursive Record occurrences, optional recursive absence, empty required strings, root absence/null/kind failures and malformed nested children alongside valid owners. Logical SQL matches independent expected envelopes and exact tokens. Receipts pin sources/captures/harness/log. Recursive numeric physical guards currently check original string storage kind, not every source-token/domain rule; full native lexical/domain and embedding publication evidence remain required alongside row trees and relationships.


### Recursive exact numeric-domain prerequisites (2026-10-07)

Recursive scalar guards now combine each captured original codec's storage-kind predicate with exact native numeric-domain checks derived from its admitted UMF descriptor. Shared domain SQL enforces finite numeric input, integer integrality/signedness/width, or decimal precision/scale. Comparator-based scalar SUM/key checks retain their original selected strategy gate before using the shared predicate; recursive domain observation selects no comparison operator or source-token grammar. Carriers keep original token text.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-recursive-domain-tests.log`). Fresh [one-call recursive-domain emissions](B-005-original-recursive-domain-native.json) and [public emissions](B-005-public-recursive-domain-native.json) each passed 20 PostgreSQL cases, including prior nested/cyclic values and complete-owner rejection of negative, fractional, uint64-overflow, NaN, Infinity and malformed nested token text alongside valid rows. Failed owners withhold SELECT execution without unsafe-cast errors. Valid exact large integers and recursive envelopes remain unchanged. Receipts pin sources/captures/harness/log. This supersedes the string-kind-only numeric-domain limitation for observed uint64 nested fixtures; recursive decimal/signed cases, selected source-token lexical grammar, native row trees, relationships and embeddings remain B-005 work.


### Original numeric map and structured member SELECT (2026-10-07)

Extended original compound admission/decoding and one-call/public Registry fixtures to the existing numeric-map and numeric-structured UMF modules. Map leaves and the optional structured numeric member retain exact uint64 type/codec correspondence; their emitted recursive guards enforce native numeric domain while result SQL preserves original text. Rust selected fixture decoding independently refuses out-of-range values.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-numeric-container-tests.log`). Fresh [one-call emissions](B-005-original-numeric-container-native.json) and [public emissions](B-005-public-numeric-container-native.json) each passed 26 PostgreSQL cases. Valid results retain uint64 maximum and values beyond JavaScript precision in literal-key maps and optional member envelopes. Required/optional absence, empty containers/strings and malformed root/leaf cases retain their prior semantics. Negative/fractional/overflow/NaN/Infinity/malformed numeric leaves alongside valid rows trigger full-owner prerequisites and withhold SELECT execution. Receipts pin sources/captures/harness/log. Recursive decimal/signed integer and source-token grammar, native row trees, relationship operations and embeddings remain B-005 work.


### Complete-state native row-tree custody observation (2026-10-07)

Captured root locations now retain the independently qualified node/scalar relation names from their original join selectors. A private complete-state observation emits every node/payload occurrence as 23 text-or-null custody cells: exact state/node/parent IDs, kind/slot/ordinal/key, node field-identity/definition/source bytes, scalar typed slots, original numeric token and codec/source bytes. FULL JOIN retains orphan/duplicate payloads for explicit downstream refusal. It does not reconstruct or publish a logical value, infer native bytes meaning, or provide a bounded database allocation guarantee.

All 86 PostgreSQL crate tests passed (`/private/tmp/weft-tree-custody-tests.log`). Captured object/edge selector fixtures passed [10 PostgreSQL cases](B-005-row-tree-custody-native.json): absent state, container-only state, descendant uint64 scalar, orphan payload and duplicate payload, for both owner kinds. Exact large values, bytea including NUL/empty bytes, NULL versus empty text and foreign-state exclusion survive transport. Malformed observations remain visible rather than disappearing. Receipt pins source/capture/harness/log. Original-graph tree admission, bounded host custody parsing, recursive row logical reconstruction and public one-call/embedding execution remain required B-005 work.


### Bounded complete-state row custody admission (2026-10-07)

Added a host-framed 23-cell bag parser with whole-input cell/copy/decoded-byte reservation before allocating output rows. Native IDs/ordinals require canonical signed bigint text; byte cells require canonical lowercase hex and retain both native spelling and decoded bytes. NULL, empty text and empty bytea remain distinct. Duplicate/orphan rows and unknown scalar kinds stay intact for the later original-topology/codec gate; custody parsing alone cannot authorize publication.

All 88 PostgreSQL crate tests passed (`/private/tmp/weft-tree-admission-tests.log`). Tests cover exact IDs beyond JavaScript precision, bytea with NUL, empty/NULL distinctions, full-bag budget refusal before copies, charged invalid-hex failure and noncanonical/overflowing IDs. Fresh [10 native bags](B-005-row-tree-custody-native.json) now record every observed cell; Rust admission checks exact cell preservation for each, including malformed orphan/duplicate observations. [Admission receipt](B-005-row-tree-admission.json) pins source/native receipt/log. Complete-tree structural/original-graph admission, logical native row reconstruction and public compiler/embedding execution remain B-005 work.


### Complete-state native tree structural admission (2026-10-07)

Added a bounded physical tree index over admitted custody rows. It reserves node work before index allocation, checks canonical native IDs/ordinals, single-state custody, unique node/payload occurrences, required original definition/source custody, scalar payload ownership and container/null payload absence. Parent shape determines legal child slot, sequence ordinals must be dense/unique, map/record slots must be unique, and iterative root traversal enforces depth and complete reachability. Children of scalar/null nodes, orphan payloads, disconnected cycles and missing roots refuse. Original UMF shape/identity/codec interpretation remains a separate gate.

All 90 PostgreSQL crate tests passed (`/private/tmp/weft-tree-index-tests.log`). All ten recorded native bags enter custody parsing; only the complete container/descendant bags for each owner kind pass tree structure. Mutated native vectors prove foreign state, duplicate ID, wrong/missing parent, wrong slot, ordinal gap/noncanonical spelling, payload ownership mismatch and disconnected-cycle refusals. Node-budget failure occurs before indexing, and bounded depth independently refuses. [Receipt](B-005-row-tree-structure.json) pins source/native receipt/log. No fresh database execution was necessary because the parser/index tests consume the exact recorded native cells. Original-graph recursive row decoding and public compiler/embedding execution remain B-005 work.


### Bounded native tree storage-body reconstruction (2026-10-07)

Added iterative reconstruction of an intermediate stored-name body under original value topology. Selected host procedures receive original topology/codec metadata and native definition/source custody, decode exact scalar values, interpret native field identities and decide missing-member validity from original presence bytes. Structured metadata references observe the same physical container; sequences preserve indexed order and maps preserve literal keys. Child work, field correspondence and output key bytes are bounded. Unknown/ambiguous identities, repeated logical fields, shape mismatch and inexact numeric/container leaf results refuse.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-row-reconstruction-tests.log`). The sequence test consumes recorded native PostgreSQL cells and preserves the uint64 maximum as text. A synthetic structured-tree mutation of those cells checks original member identity/absence procedures and independently observed structured/record metadata. Work limits refuse before queued child or field-identity callbacks. [Receipt](B-005-row-value-reconstruction.json) pins source, test log and original native receipt. This is intermediate reconstruction: original property admission coupling, native scalar/source qualification, logical envelopes and emitted public recursive row SQL/embedding evidence remain B-005 work.


### Original row-property reconstruction admission (2026-10-07)

Added the original-property entry point for intermediate native reconstruction. Before host observation/decoding it verifies captured binding basis and property ownership, requires admitted row-home access, checks original graph JSON and UMF descriptor correspondence, rechecks captured member-presence definitions and every scalar codec artifact, and derives topology from that checked graph. Selected native field/source/codec procedures still own interpretation; this gate does not invent their meanings or publish result envelopes.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-property-reconstruction-tests.log`). The existing original UMF application fixture now couples its admitted native-home string property and independently lowered access to synthetic complete text custody, preserving Unicode/trailing spaces. Substitution of the props binding refuses before either observer or decoder. [Receipt](B-005-row-property-reconstruction.json) records source/log hashes and scope. Public recursive row emission, logical-envelope integration and fresh embedding evidence remain B-005 work. No new native SQL execution is claimed for synthetic text custody.


### Shared logical-member bridge for admitted native bodies (2026-10-07)

Added a present-tree logical decoding entry point that first applies original row-property admission and selected native procedures, then uses the existing original UMF logical-member decoder for authored names and member presence envelopes. It passes only already-qualified exact string/boolean/null leaves into that bridge. Both passes consume the same work budget. Root absence/null publication still belongs to the separate root-presence gate; native numeric/source interpretation remains the selected procedure's responsibility.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-row-logical-bridge-tests.log`). Original admitted string custody survives both passes with Unicode and trailing spaces; a one-node shared budget refuses the second pass after native reconstruction. [Receipt](B-005-row-logical-bridge.json) pins source/log hashes and limits. Recursive native structured corpus evidence, SQL emission and public embedding integration remain B-005 work; this does not claim fresh database execution or public recursive row support.


### Original compound native-home decoding corpus (2026-10-07)

The original UMF compound corpus now independently admits complete-value-tree row homes using the original graph/descriptors, captured leaf/member definitions and selected physical inventory/join mapping. Seven fixture trees cover strings with Unicode/trailing spaces, empty/literal map keys, optional structured members, cyclic metadata with finite values, nested sequences and exact uint64 maxima in maps/records. A fixture procedure selects exact JSON bytes for native field identities; its scope is explicit rather than inferred as Truss's adopted encoding.

The private complete-state custody SQL passed [14 fresh PostgreSQL round trips](B-005-original-recursive-row-native.json), one object and one edge physical owner per fixture. Every one of the 23 cells per native row equals its input, including full codec/definition bytes and numeric tokens. Rust consumes the recorded native cells under the independently admitted object property/access, checks custody/tree structure, reconstructs storage bodies and produces logical results equal to the props-home decoder. All 92 PostgreSQL crate tests passed (`/private/tmp/weft-original-recursive-native-tests.log`). [Decoder receipt](B-005-original-recursive-row-decoding.json) pins fixtures/source/log/native evidence. This does not establish edge property semantic admission, an adopted field encoding, full source-token grammar or public recursive row SQL/embedding execution; those remain B-005 work.


### Original native compound semantic refusals (2026-10-07)

The seven original native-home fixture paths now return diagnostics for selected definition/source custody mismatch and uint64 overflow instead of relying on fixture assertions/panics. Mutations of recorded native cells establish 21 refusals: definition and source changes for every fixture, overflow in nested sequence/map/structured numeric values, unknown field identities in three record graphs, and a physically valid empty structured root missing its required street member. The latter passes physical structure and is refused by original UMF logical member presence semantics.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-recursive-refusal-tests.log`). [Receipt](B-005-native-recursive-refusals.json) pins source/log/native cells and exact refusal counts. These are host decoding mutations of already recorded PostgreSQL custody, with selected fixture interpretation; no new database or public recursive row SQL/embedding support is claimed. SQL lowering must preserve equivalent semantic refusals before publication.


### Original native topology lowering packet (2026-10-07)

Added parameterized lowering preparation for native row trees. Original row property/access, graph/descriptors, member presence and scalar codec custody are checked before selecting field bytes. The trusted backend procedure encodes original field identities; duplicate encoded identities within a record refuse. The packet retains root/topology, exact codec/member-presence bytes, field identities and stored slots, with typed parameter origin linked to the original property. Parameters commit atomically only after the complete packet and private custody SQL are ready.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-tree-mapping-tests.log`). Seven independently admitted original compound row fixtures produce one typed topology slot; the three record graphs refuse aliasing field encodings, and all seven refuse a substituted props binding before invoking the encoding callback. Every refusal leaves parameter slots unchanged. [Receipt](B-005-native-tree-mapping.json) pins source/log and scope. Recursive SQL assembly must still turn these lowering inputs into the defined logical carrier with complete structural/semantic prerequisites; no public recursive row or fresh native/embedding qualification is claimed.


### Original native topology/path SQL walk (2026-10-07)

Added recursive SQL pairing of complete native custody to original graph nodes and stored-member paths. Structured graph references use the same physical container; arrays/maps select original item nodes and records match explicitly selected native field bytes. Metadata is one typed slot appended after existing owner/home slots. The emitted intermediate observation retains every matched custody row, its original node/path/depth and partial unmatched/duplicate/shape facts; it is not a public logical carrier or complete integrity proof.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-tree-walk-tests.log`). [28 fresh PostgreSQL cases](B-005-native-tree-walk-native.json) execute exact captured original access/parameter SQL: valid, orphan, duplicate and wrong-shape controls for every compound fixture. Independent stored-body traversal checks every valid path/depth and exact scalar leaf, including Unicode, literal/empty map keys, cyclic metadata and uint64 tokens. [Receipt](B-005-native-tree-walk.json) pins source/log/native evidence. Complete physical/source/codec validation, logical envelope assembly, owner-wide prerequisites and public embedding integration remain B-005 work; the bounded-depth walk is not a database allocation bound.


### Native stored/logical body SQL assembly (2026-10-07)

Added breadth-first stored-body SQL assembly over the original native topology/path walk. Empty maps/records and array skeletons precede child assignments, so numeric array indexes remain correct independently of lexical path order. Selected backend leaf SQL returns exact JSONB from native custody; the existing original UMF recursive SQL bridge then establishes logical member names/envelopes and its codec/presence/domain integrity. Parameters stage across topology, selected leaf lowering and the shared logical bridge, committing only after complete success.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-tree-body-tests.log`), including unchanged parameters after a selected leaf-lowering refusal for each compound fixture. [11 fresh PostgreSQL cases](B-005-native-tree-body-native.json) prove all seven stored/logical bodies equal independent expected fixtures and separately report false logical integrity for three uint64 overflows and a missing required structured member. [Receipt](B-005-native-tree-body.json) pins source/log/native evidence. This is intermediate assembly; complete native structural/source qualification, owner-wide prerequisites, root publication presence and public projection/embedding wiring remain B-005 work. Selected fixture scalar/source meanings do not imply an adopted Truss production codec, and SQL allocation remains unqualified.


### Native tree structural SQL prerequisite (2026-10-07)

Added SQL structural integrity alongside native body assembly. It retains the independently selected root-location prerequisite, requires one reachable nonempty single-state tree with unique node and storage-path occurrences, legal root/child slots, dense unique sequence ordinals, original definition/source custody, scalar payload ownership/codec-source custody and container/null payload absence. Original topology pairing must cover every custody row and match every physical shape. The observation row-count/depth limits do not establish bounded database allocation. Typed scalar slot/source meaning and exact native/token correspondence remain separate selected procedures.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-tree-structure-tests.log`). [56 native structural cases](B-005-native-tree-structure-native.json) cover seven original compound trees plus 49 orphan/duplicate/slot/container-payload/source-custody/ordinal-gap/unknown-field controls. [11 separate body cases](B-005-native-tree-structure-body-native.json) preserve all expected stored/logical values and prove logical overflow/missing-required-member failures can retain valid physical structure. Dedicated capture/harness files preserve prior body evidence. [Receipt](B-005-native-tree-structure.json) pins source/log/native evidence. Native source/typed-leaf qualification, owner-wide prerequisites, root publication presence and public projection/embedding wiring remain B-005 work.


### Explicit selected native/source SQL prerequisites (2026-10-07)

Added guarded native body lowering with independently selected leaf rendering/integrity and node definition/source procedures. Every visited node predicate must be TRUE; NULL is a refusal. These guards remain separate from logical domain/member integrity and physical tree custody. Selection and all parameter additions stage atomically, including a late source-procedure refusal after body/metadata preparation. The concrete corpus procedures explicitly select original codec bytes, empty source bytes, closed text/integer payload slots, canonical unsigned decimal token spelling and native numeric/token correspondence; they are fixture meanings, not inferred Truss adoption.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-tree-guarded-tests.log`). [65 fresh PostgreSQL selected-procedure cases](B-005-native-tree-guarded-native.json) cover valid fixtures and changed definition/node-source/leaf-codec/leaf-source/kind, extra typed slot, missing source, native/token mismatch, noncanonical token and invalid token. Native/token mismatch and noncanonical tokens retain true logical integrity yet independently refuse native integrity. Missing scalar source also refuses structural custody; NULL predicates cannot disappear from bool aggregation. Seven late source-procedure refusals preserve parameter slots. [Receipt](B-005-native-tree-guarded.json) pins source/log/native scope. Owner-wide prerequisites, root publication presence and public result projection/embedding remain B-005 work; no production codec or database allocation qualification is implied.


### Native compound projection and complete-owner prerequisite (2026-10-07)

Added original native recursive result projection under explicitly selected identity/source/scalar procedures. The text carrier contains the defined absent/value envelope and original logical member representation; required versus optional root absence derives from the admitted original descriptor. Compound explicit null has no admitted meaning and cannot pass the guarded tree path. A separate complete-owner violation query combines root-location, full native structure, logical integrity and selected native/source predicates; it shares the exact projection parameters and commits their additions atomically.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-tree-projection-tests.log`). [28 fresh PostgreSQL cases](B-005-native-tree-projection-native.json) cover valid carriers, absent-root policy, hidden corruption and foreign-owner scope for each original compound fixture. The result query filters one owner and applies LIMIT 1; a corrupt second owned record still fails its independent prerequisite. A foreign owner type does not widen selected scope. [Receipt](B-005-native-tree-projection.json) pins source/log/native evidence. Public registry/one-call compiler and Python/WASM integration remain B-005 work; hosts still own qualified read-context/prerequisite execution before publication, and fixture procedures do not claim adopted production codecs.


### Public original-registry native compound projection (2026-10-07)

Original complete-value-tree property admissions can now hold explicitly selected trusted Rust field-identity/scalar/source procedures privately. Selection rechecks original graph/descriptor, member-presence and leaf codec custody and refuses other physical home meanings. Parameter-aware projection and payload-observation paths use that selected state for native compound reads; missing selection returns a named capability refusal. Models cannot load procedures. Both original one-call SELECT assembly and the public backend registry therefore emit native compound result carriers with independent structural and complete-owner payload obligations.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-tree-registry-tests.log`). Seven original native compound fixtures compile through both entry points with exact SQL/parameter parity. [21 fresh PostgreSQL public compiler cases](B-005-native-tree-public-native.json) execute the captured SELECT and every emitted SQL prerequisite with valid, corrupt-owned and foreign-owner controls, preserving exact logical envelopes and selected owner scope. [Receipt](B-005-native-tree-public.json) pins source/log/native evidence. This proves the selected native recursive projection subset; fresh original-registry Python/WASM qualification, remaining B-005 numeric/page/relationship corpus and host execution/read-context gates are still required. Fixture procedure meanings do not establish adopted Truss production codecs.


### Missing native procedure selection refusal boundaries (2026-10-07)

The seven original compound fixtures now first lower physical accesses without any selected native procedures. Result projection, complete-owner payload observation and original one-call compilation each refuse with a capability diagnostic. Projection/observation failures preserve parameter slots; one-call compilation refuses before operand rendering. Only after these checks does each fixture select its trusted procedures and retain its established public compilation behavior.

All 92 PostgreSQL crate tests passed (`/private/tmp/weft-native-selection-refusal-tests.log`), including 21 selection refusals and 14 parameter-rollback checks. [Receipt](B-005-native-selection-refusals.json) pins source/log and the embedding inspection. `weft-python` and `weft-wasm` call `weft-runtime::compile_json`; that runtime currently registers the fixed candidate backend. Original-registry serialized admission/composition in shared Rust remains implementation work before original-native Python/browser claims. This is not an external Truss adoption dependency. No new database execution or embedding support is claimed by these Rust refusal checks.


### Shared Rust validated registry-composition hook (2026-10-07)

Added an optional trusted request-local registry factory to the public pure Rust compiler. The existing default compile method remains unchanged. Factory invocation follows envelope/resource checks, original binding JSON/digest admission, UMF catalog preparation and SQL name/type/plan resolution; it receives that admitted catalog/plan plus original target bytes/pin and candidate policy. Its returned registry performs backend binding/capability/emission admission through the same compiler transport. Model content never loads executable code, and composition remains host-provided pure Rust admission/registration.

All 130 core/PostgreSQL tests passed (`/private/tmp/weft-composition-factory-tests.log`). Seven native compound serialized requests use an already-admitted owned registry through the hook and match direct registry SQL, parameters, columns, obligations, logical plan and binding digest while retaining candidate qualification. Digest mismatch, unknown field and malformed SQL never invoke composition (21 refusals); seven explicit factory failures invoke once and expose no partial result fields. [Receipt](B-005-composition-factory.json) pins source/log and limits. Built-in serialized original-admission composition and fresh Python/browser qualification remain B-005 work; this shared hook does not itself provide that adapter or claim fresh database/embedding execution.


### Reusable original-byte backend admission batch (2026-10-07)

Added exact UMF Record/member identity lookup and a pure original-backend admission batch. It verifies binding digest/grammar, independently admits selected Record sources, derives each original member's descriptor graph from the selected UMF catalog, admits original property value/home bytes and selected native procedures, and verifies each property against its independently admitted owner. Duplicate Record/property selections and missing owner admissions refuse before an OriginalBackend is returned. Physical/codec/presence/comparator interpretations remain trusted Rust selections; no model-authored executable code or SQL is accepted.

All 130 core/PostgreSQL tests passed (`/private/tmp/weft-original-admission-batch-tests.log`). Seven native compound public backends now use the batch. Serialized factory tests build new admissions from request bytes on each of two invocations per fixture and compare identical full responses with the direct registry; no cached admitted registry is returned. Wrong Record revision/member document, digest mismatch, duplicate Record/property selections and missing independent owner each refuse. [Receipt](B-005-original-admission-batch.json) pins source/log and limits. Built-in Python/WASM runtime composition and fresh embedding evidence remain B-005 work; this reusable Rust admission path does not claim new database or embedding execution.


### Owned pure Rust original-backend composition (2026-10-07)

Added owned Record/property selections and an original-backend Configuration that holds admitted codec/presence/comparator meanings, original physical associations and statically selected native procedures. Its serialized compile method uses the validated factory hook and re-admits every request's original bytes; it caches no admitted backend. Procedure-bearing configuration intentionally has no serde executable-name interface. Comparator definition clones retain private type/operation custody for request-local construction. Other backend IDs cannot be silently substituted.

All 130 core/PostgreSQL tests passed (`/private/tmp/weft-owned-composition-tests.log`). Seven native compound configurations match full factory responses, repeat deterministically and refuse backend substitution without SQL. `cargo check -p weft-postgresql --target wasm32-unknown-unknown` passed (`/private/tmp/weft-owned-composition-wasm-check.log`), proving target compilation only. [Receipt](B-005-owned-composition.json) pins source/log and limits. Python/WASM entry-point adapter integration and real embedding execution remain B-005 work; target compilation does not establish browser behavior, production codec adoption or new database qualification.


### Conformance embedding composition captures (2026-10-07)

Captured the seven owned original configurations as explicit conformance-only metadata paired with their serialized compile requests/full expected responses. The packets preserve original inventory, leaf/source/domain artifacts, member-presence definitions, native join artifact custody, selected physical relations/columns and graph profiles. Procedure pointers are absent; the forthcoming harness must supply its explicit fixture procedures in compiled Rust. This packet is test data, not a released host registration protocol or model-driven plugin format.

All 130 core/PostgreSQL tests passed with captures enabled (`/private/tmp/weft-original-composition-capture-tests.log`). The persisted independent Python custody checker validates canonical base64/digests for inventory, codec/join artifact paths, member definitions and binding/response pins across all seven pairs. [Receipt](B-005-composition-captures.json) pins captures/source/log/checker. Runtime composition and real native Python/browser execution remain B-005 work; capture production and Python custody checking do not qualify either embedding.

## Original composition through native embedding ports (2026-10-07)

A new explicitly optional `test-original` feature reconstructs the seven captured
native compound configurations in Rust. It rechecks original artifact digests,
reconstructs selected leaf/member/row procedures, and re-admits catalog and
binding custody for every compile request. Its exact binding allowlist and closed
fixture procedures are conformance machinery, not a released generic backend
registration protocol or production codec support.

`cargo test -p weft-core -p weft-postgresql -p weft-runtime --features
weft-runtime/test-original` passes 131 tests. The runtime check covers all seven
full expected responses, repeated fresh composition, candidate opt-out and
rehashed unsupported binding cuts.

The actual Python extension, built with maturin 1.9.6 / Rust 1.90 / Python
3.12.14 using `--features test-original`, passes
`tests/truss-postgresql/original-embedding-check.py` with subprocess entry points
disabled. Seven original responses match the captures; seven deterministic
repeats and 21 negative requests (candidate opt-out, unsupported cut, wrong
digest) pass. Wheel SHA256:
`55e6cc8352313ed0330e6c23aef1e83163677c25fbf42ec21913ae4424ea506d`.
This local wheel is abi3 cp39, macOS 11 ARM64; other platforms remain untested.

A fresh `weft-wasm` build for `wasm32-unknown-unknown --features test-original`,
with wasm-bindgen 0.2.105 web glue, passes the same 28 requests byte-for-byte
against the Python responses in real Chromium 153.0.8010.12 / Playwright 1.62.1.
The existing `tests/compile/browser-check.mjs` runs with generated
`target/b005/original-embedding/{cases,reports}.json` as corpus/reports and
`WEFT_PROBE_JS` / `WEFT_PROBE_WASM` pointing to the fresh web output. Compile
network APIs are disabled after loading; no Node globals or WASI imports are
present. The wrapper's transport and trap retirement checks also pass.
Browser launch required sandbox escalation for macOS Mach services; the
compiler receives no additional host capabilities.

Receipts are in `tests/truss-postgresql/evidence/original-embedding/`.
WASM SHA256: `6d10c9c1cc99bc8fd347393d58ab0d3e88278cec7d533a5bb6dc3fb232b6f12a`.
These checks establish embedding parity for the pinned seven configurations.
They do not close B-005: broader original scalar/relationship/query evidence,
generalized registration and qualification remain unfinished.

## Original numeric row SUM and key page (2026-10-07)

The original decimal/unsigned comparator tests now independently admit native row
homes from the same UMF scalar definitions used by their props-home cases.
They compile through one-call original registry lowering and public Backend
registration, checking exact SQL/parameter parity and result metadata against
the original props result contract. The unsigned page uses the authored stable
key mapping and emits its separate complete-owner uniqueness prerequisite.
Selected fixture lowering reads the native numeric slot; SUM remains owned by
the admitted comparator, rather than the fixture expression callback.

Captures `original-{decimal,integer}-row-sum.json` and
`original-integer-row-page.json` are generated by the existing numeric tests
with `WEFT_ORIGINAL_{NUMERIC,INTEGER}_CAPTURE_ROW_SUM` and
`WEFT_ORIGINAL_INTEGER_CAPTURE_ROW_PAGE` set to absolute output paths.
`tests/truss-postgresql/original-numeric-row-native.py` executes each exact
captured SELECT and every emitted SQL prerequisite in PostgreSQL 17.9 using
isolated temporary tables in the owned fixture container.

[19 native cases](B-005-original-numeric-row-native.json) cover empty/exact SUM
and pages, uint64 maximum and values beyond JavaScript's exact integer range,
26-digit decimal magnitudes at authored scale 2, overflow, fractional unsigned
values, excess decimal scale, wrong codec, required absence and duplicate page
keys. Prerequisites refuse corrupt cases before the harness runs their result
query. Valid unsigned pages retain numeric ordering and exact text carriers;
valid decimal SUM retains the authored fractional value. Unrelated object types
remain outside selected owner scope. Fixture source bytes remain synthetic;
these checks do not qualify production codec/source interpretation, composite
cursor pages, relationships or host transaction/policy enforcement.

All 131 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-original-numeric-row-full-tests.log`); formatting and HELIX
artifact checks pass. No new Python/browser numeric-row support is claimed by
this native database run. B-005 remains open for the remaining full query corpus.

## Original directed relationship admission/access (2026-10-07)

`relationship_definition::RelationshipAdmission` now verifies a selected
relationship against original pinned UMF source/definition bytes and a fresh
Catalog traversal resolution. The complete resolved read is compared, including
forward/inverse direction, both independently authored keys, multiplicity and
lifecycle. Both endpoints require independent original Record admissions and
exact key mapping validation. Physical edge roles come from trusted Rust
selection over the exact binding layout inventory; foreign-relation columns or
aliased physical roles refuse.

Its parameterized access primitive correlates native object ID **and type**,
selects the relationship discriminator and both endpoint types, and returns the
correct opposite physical endpoint for either direction. It preserves a private
binding/traversal cut and stages all parameters before committing them. Logical
key tokens do not substitute for native IDs.

The new Rust test covers both directions plus substituted lifecycle,
multiplicity, endpoint revision and keys, missing independent endpoint Records,
foreign/aliased columns and changed mapping cuts. Wrong-direction access refuses
without changing existing parameters. Captures can be regenerated with
`WEFT_ORIGINAL_RELATIONSHIP_CAPTURE` set to an absolute output path when running
`cargo test -p weft-postgresql original_directed`.

[Four PostgreSQL 17.9 cases](B-005-original-relationship-access-native.json)
execute exact captured forward/inverse access SQL with empty and populated
synthetic layouts. Controls include reused native IDs across object types,
wrong relationship discriminators, wrong source/target types and reversed
endpoints. Only the two expected directed edges are returned.

This is reusable original custody/access implementation, not complete public
relationship query lowering. `HAS_RELATED` and `RELATED_KEYS` still require
integration, separate target property access, key projection, uniqueness,
bounded expansion and complete-owner prerequisites before B-005 can close.
No relationship cardinality enforcement, association Record or production
storage compatibility is claimed by these access tests.

All 132 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-original-relationship-full-tests.log`); formatting and HELIX
artifact checks pass. This run makes no new relationship embedding claim.

## Scoped original relationship target preparation (2026-10-07)

Original source preparation now has a trusted compiler-assigned target scope.
Ordinary owner aliases stay unchanged; target owner aliases use
`weft_related_{scope}_scan_{index}` and row-home state/node/scalar aliases use
disjoint 1,024-index ranges. Scope and access-count bounds refuse before parameter
mutation. This prevents target subqueries from shadowing outer native owner or
row aliases during correlation; model metadata cannot supply aliases or SQL.

`RelationshipAdmission::prepare_target` checks its private original traversal
and binding cut, independently verifies the target Record's authored key
mapping, and constructs a key-reading target plan from exact original field
descriptors. The plan and its prepared access custody travel together for
subsequent lowering. Source and target keys are not zipped to each other.
Numeric row tests prepare two distinct scopes, verify context/parameter custody,
check all physical alias inventories and refuse an out-of-range scope atomically.
An original inverse Orders-to-Customer read prepares its admitted uint64 target
key through the new method; a substituted target key refuses without changing
existing parameters.

[Four fresh PostgreSQL cases](B-005-scoped-target-native.json), executed by
`tests/truss-postgresql/scoped-target-native.py`, check empty and populated
native-row target access in two scopes. Each exact captured query correlates
outer object ID and type and returns unchanged `9007199254740993` and uint64
maximum text carriers. A reused native ID in an unrelated object type does not
multiply results. Capture regeneration uses absolute
`WEFT_ORIGINAL_INTEGER_CAPTURE_SCOPED_0` / `_1` paths with the original unsigned
property test. These captures exercise scoped access, not final relationship
publication.

All 132 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-scoped-target-full-tests.log`). Full `HAS_RELATED` /
`RELATED_KEYS` result and preflight assembly remains required B-005 work; no
new public relationship or embedding support is claimed by target preparation.

## Original relationship subquery lowering (2026-10-07)

`relationship_lowering::lower` now composes the original admitted directed edge
and separately scoped target-key access into existential lookup and bounded key
list expressions. Each target key component requires its original selected
comparator, exact type and key/equality operation; bounded lists also require
ordering. Target numeric domain and key uniqueness checks observe complete
selected target owners independently of the result filter or lookahead. Native
key carriers remain exact text inside the tuple JSON arrays.

`HAS_RELATED` compares every target key component against the exact typed
literal/parameter tuple and produces EXISTS, preserving source cardinality.
Named parameters retain one exact value/type slot and original name/span.
`RELATED_KEYS` supports the frontend's 1..1000 bound, orders by the complete
key, preserves duplicate-edge tuple multiplicity and probes bound+1 rows. Bound,
probe and multiplicity values use typed parameter slots. Result lists contain
`items` and an exact `truncated` marker; empty lists have false truncation.
Failed key arity and invalid bounds leave caller parameters unchanged.

Original relationship admission additionally emits full selected-edge endpoint
checks and per-owner multiplicity checks independent of query filters/limits.
The endpoint check refuses missing native owners and mismatched endpoint types
for the selected relationship. Cardinality uses the authored target multiplicity
for forward reads and source multiplicity for inverse reads. Numeric min/max
limits retain unsigned text rather than passing through floating point.

[14 PostgreSQL 17.9 cases](B-005-relationship-lowering-native.json), executed by
`tests/truss-postgresql/relationship-lowering-native.py`, cover original inverse
Orders-to-Customer lookup/list expressions with independently admitted native
uint64 target properties. Exact cases retain uint64 maximum and
`9007199254740993`, numeric ordering, duplicate-edge bag multiplicity, true
lookahead truncation and empty results. Hidden unconnected target overflow,
wrong codec, duplicate target logical keys, dangling targets and wrong endpoint
types each fail at least one prerequisite before the harness executes the result
query. The selected fixture multiplicities are 0..*; finite min/max branches
still require separate native qualification. Capture regeneration sets absolute
`WEFT_ORIGINAL_RELATIONSHIP_LOWERING_CAPTURE` when running the original unsigned
property test.

All 132 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-relationship-lowering-full-tests.log`); formatting and HELIX
artifact checks pass. This is reusable subquery lowering, not yet public original
Backend SELECT integration. The outer compiler still needs to incorporate these
expressions/checks and qualify both directions, composite keys, both storage
homes and real embedding ports. Hosts remain responsible for complete read
context and prerequisite enforcement. No production compatibility is claimed.

## Public original Backend relationship integration (2026-10-07)

`OriginalBackend::with_relationships` accepts independently admitted original
relationship selections keyed by exact identity and traversal direction, and
refuses substituted binding cuts or registration keys. Configured relationship
capabilities retain candidate qualification. Missing selections refuse rather
than falling back to the fixed test backend.

The shared original SELECT assembler now invokes the relationship renderer for
HAS_RELATED filters and RELATED_KEYS projections. It verifies the outer scan's
original owner/key access, assigns target scopes independently and incorporates
all target payload, numeric/key, edge endpoint and cardinality observations into
the returned host SQL obligations. Existing relational/projection entry points
retain their previous behavior and refuse relationship reads without their
selected renderer. The same trusted native operand lowering handles ordinary
and relationship target accesses.

The unsigned fixture now independently admits both original endpoint Records
and both native uint64 key properties at one final binding cut. Two queries are
parsed/resolved from Weft SQL and compiled through public Backend registration:
COUNT with an inverse HAS_RELATED named parameter, and a keyed Orders page with
an inverse RELATED_KEYS projection. They also pass the serialized
`Compiler::compile_json` envelope with full SQL, parameter, result-column and
obligation equality against direct registry compilation. Captures are
`original-relationship-public.json` and
`original-relationship-public-transport.json`; regenerate with absolute
`WEFT_PUBLIC_ORIGINAL_RELATIONSHIP_CAPTURE` and
`WEFT_PUBLIC_ORIGINAL_RELATIONSHIP_TRANSPORT_CAPTURE` paths while running the
original unsigned property test.

[18 PostgreSQL 17.9 cases](B-005-relationship-public-native.json), executed by
`tests/truss-postgresql/relationship-public-native.py`, execute each exact public
emission and every emitted SQL prerequisite. Valid cases prove existential
COUNT preserves two matching source rows despite multiple/duplicate edges, and
keyed pages retain exact numeric source identities, ordered target key tuples,
duplicate-edge multiplicity, lookahead truncation and empty lists. Hidden source
and target overflow, wrong codecs on either endpoint, duplicate target logical
keys, dangling targets and wrong endpoint types each refuse before result
execution. Candidate/profile fixture evidence does not establish production
codec or host transaction/policy implementation.

All 132 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-public-relationship-full-tests.log`). This establishes the
selected original inverse/native-key public Rust subset. Forward/composite-key
and alternate-home coverage, finite multiplicity native cases, owned embedding
composition and actual Python/browser relationship execution remain B-005 work.

`cargo check -p weft-postgresql --target wasm32-unknown-unknown` also passes
(`/private/tmp/weft-public-relationship-wasm-check.log`). This is target
compilation evidence, not browser execution. Formatting and HELIX checks pass.

## Forward traversal, source key uniqueness and finite multiplicity (2026-10-07)

The public original fixture registers both directions against the same binding
cut and independently admitted native endpoint keys. Forward Customer-to-Orders
COUNT/HAS_RELATED and keyed RELATED_KEYS reads now resolve from SQL, compile
through Backend registration and match serialized compiler responses. The
transport capture contains all four forward/inverse queries; separate forward
SQL captures preserve independent native verification.

The forward corpus exposed a real source-key gate gap: a HAS_RELATED count did
not observe duplicate source logical keys unless its outer query was a keyed
page. Original SELECT lowering now collects complete source keys from all
selected relationship predicates/projections and emits owner-wide uniqueness
checks, independently validating each key against its original Record mapping.
It deduplicates checks by source occurrence/key identity and preserves existing
page behavior. Source key checks retain guarded exact numeric correspondence;
no query filter or LIMIT narrows their owner scope.

[18 forward PostgreSQL cases](B-005-relationship-forward-public-native.json)
and the [18 refreshed inverse cases](B-005-relationship-public-native.json)
pass. Forward cases prove existential counts across three Customer owners,
exact ordered Orders key tuples, duplicate-edge multiplicity, lookahead and
empty results. The duplicate Customer key case now refuses even for a count;
source/target overflow, codecs, endpoint type and dangling-target controls
continue refusing before results are executed. Harnesses execute every emitted
public SQL prerequisite independently of result queries.

A new original-custody fixture changes the authored source multiplicity to 1..2
and target multiplicity to 0..1, refreshes all existing original source artifacts
and model digests, and re-admits both Records and relationship directions.
[Eight PostgreSQL finite cases](B-005-finite-relationship-native.json) prove
forward checks use target-side bounds while inverse checks use source-side
bounds. A forward maximum violation, inverse maximum violation and inverse
minimum violation are independently distinguished; valid layouts pass in both
directions. This qualifies the finite prerequisite expressions over original
admissions, not an additional standalone public finite-query/embedding corpus.

New harnesses: `relationship-forward-public-native.py` and
`finite-relationship-native.py` under `tests/truss-postgresql/`. Captures regenerate
using absolute `WEFT_PUBLIC_ORIGINAL_FORWARD_CAPTURE` and
`WEFT_FINITE_RELATIONSHIP_CAPTURE` paths with the original unsigned and finite
multiplicity Rust tests, respectively. All 133 core/PostgreSQL/runtime tests
pass with `weft-runtime/test-original`
(`/private/tmp/weft-forward-finite-full-tests.log`); formatting and HELIX checks
pass. Composite endpoint keys, alternate homes and owned Python/browser
relationship composition remain B-005 work. No production qualification is
inferred from the synthetic PostgreSQL layouts.

## Owned original relationship composition (2026-10-07)

`original_admission::Configuration` now includes owned relationship selections:
binding index, explicit direction, selected profile, exact inventory and trusted
physical edge roles. Like other owned selections these are Rust host inputs,
not a serde plugin-loading protocol. Each `backend` invocation reconstructs
Record/property admissions, resolves the authored relationship name from the
current original definition, independently admits each direction and registers
only after the whole batch succeeds. Duplicate directions and substituted
original registration cuts refuse.

The four forward/inverse public native-key queries now pass through this owned
configuration's `compile_json` factory, comparing their complete responses with
direct public registry output and repeating deterministically. A duplicate
relationship selection is inserted for each query and refuses without SQL;
removing it restores the valid configuration. Existing seven compound owned
configurations explicitly select no relationships and keep their prior scope.

All 133 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-owned-relationship-tests.log`).
`cargo check -p weft-postgresql --target wasm32-unknown-unknown` passes
(`/private/tmp/weft-owned-relationship-wasm.log`), proving compilation only.
Actual native Python/browser relationship adapter composition and execution
remain required B-005 work; neither is claimed by these Rust checks. This adds
no fresh PostgreSQL execution or production storage qualification.

## Relationship embedding composition capture (2026-10-07)

A test-only `Configuration::conformance_capture` preserves owned Record/property,
comparator and directed relationship selection inputs. It records original
artifact bytes/digests and trusted physical selectors, plus the presence of a
native-tree procedure, without serializing executable pointers or callable
names. `original-relationship-composition.json` captures the independently
admitted two native uint64 endpoint keys, their original comparators and both
relationship directions. This remains conformance data, not a released plugin
registration protocol. Capture regeneration uses absolute
`WEFT_ORIGINAL_RELATIONSHIP_COMPOSITION_CAPTURE` with the original unsigned
property test.

The persisted independent checker `check-composition-custody.py` now verifies
eight configurations and eleven complete compiler responses: seven compound
pairs plus four relationship requests at one binding cut. It checks canonical
base64, every comparator/codec/join original-artifact digest, both relationship
physical role inventories, request/response binding pins, and comparator value
and source-domain definitions against the selected original property artifacts.
[Receipt](B-005-relationship-composition-capture.json) pins capture/checker/source
and test log bytes.

All 133 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-relationship-composition-capture-tests.log`); formatting and
HELIX checks pass. Runtime conformance loading of the new relationship capture
and actual Python/browser execution remain the next embedding work. No new
native database execution, embedding support or production compatibility is
claimed by this data-custody checkpoint.

## Original relationship native Python/browser execution (2026-10-07)

The optional `test-original` runtime now includes the one exact captured native
uint64 relationship binding alongside the seven compound presets. Shared Rust
reconstructs original comparator artifacts/types from the current Catalog,
selects the closed fixture numeric operand procedure, respects scalar-vs-tree
procedure selection and re-admits both relationship directions for every compile.
The binding allowlist remains exact. No executable procedure is loaded from
model content; this is conformance-only composition, not general released
backend registration or production codec compatibility.

All 134 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`
(`/private/tmp/weft-relationship-runtime-tests.log`). A runtime regression matches
all four full relationship responses and deterministic repeats. The real native
Python extension, freshly built by maturin 1.9.6 / Rust 1.90 / Python 3.12.14
with `--features test-original`, passes the expanded
`tests/truss-postgresql/original-embedding-check.py`: 11 complete expected
responses, 11 deterministic repeats and 33 candidate-opt-out/digest/unsupported-
binding refusals, with subprocess entry points disabled. The wheel is local
macOS 11 ARM64 abi3 cp39; other platforms remain unqualified.

A fresh WASM build with the same feature and wasm-bindgen 0.2.105 web glue passes
all 44 requests byte-for-byte against Python in Chromium 153.0.8010.12 /
Playwright 1.62.1. The existing browser harness uses generated
`target/b005/original-embedding/{cases,reports}.json`, fresh web JS/WASM paths and
the same compile wrapper. Network APIs are disabled after initialization; only
string/error/memory glue imports exist, and no Node globals or external network
requests appear. Wrapper transport/trap-retirement checks also pass. Chromium
requires sandbox escalation for macOS process services; compiler IO capabilities
are unchanged.

[Receipt](B-005-relationship-embedding.json) pins source, logs, summaries and
binary hashes; summaries are in
`tests/truss-postgresql/evidence/relationship-embedding/`. Wheel SHA256:
`93dea07c8d011214d64d1e5a15053b2350f3671cf15a9f3b6a13b5fac682ff84`.
WASM SHA256:
`be5a1a1624f78dedea2793d788b9af096b941be7f28f9cce9aa64199c8c4b222`.
Formatting, HELIX and eight-configuration/eleven-response custody checks pass.
This establishes actual embedding behavior for the selected forward/inverse
native-key subset. Composite relationship keys, alternate homes, broader query
coverage and generic released composition remain B-005 work; no fresh database
or production compatibility claim is added by this embedding run.

### Original relationship props-home qualification (2026-10-07)

The original unsigned-key fixture now compiles both directed HAS_RELATED and
RELATED_KEYS reads over JSONB props homes using the same original UMF key and
comparator definitions as the native-row fixture. Full logical plans and result
column contracts agree across homes. This is public Rust Configuration/Compiler
composition; no additional native Python or browser qualification is claimed.

`tests/truss-postgresql/props-relationship-native.py` executes all emitted SQL
prerequisites on PostgreSQL 17.9 before permitting result queries. Its 44 cases
cover both directions and operations, empty owners, exact unsigned 64-bit and
above-JavaScript-safe keys, ordered bags with duplicate edges, bounded lookahead,
hidden source/target overflow, duplicate keys, missing keys, wrong JSON kinds,
invalid numeric text, fractional unsigned keys and invalid edge endpoints.
Corrupt fixtures report violations; their result SQL is not executed. Receipt:
[B-005-props-relationship-native.json](B-005-props-relationship-native.json).

The invalid-text case found an unsafe cast in target-key uniqueness SQL. Numeric
uniqueness expressions now use the independently admitted comparator domain
predicate to guard their casts. This mirrors source-key uniqueness and makes
this prerequisite safe even when the target numeric domain is invalid. Captures
for the existing native relationship paths have been regenerated. Their 14
component, 18 inverse and 18 forward PostgreSQL cases pass again. Previous
embedding receipts retain their original source/binary scope; refreshed Rust
runtime full-response parity covers the changed prerequisite, but fresh Python
and browser builds for this change have not been claimed.

These are synthetic fixtures over pinned draft realization contracts. Mixed
homes, composite endpoint keys, broader original scalar semantics and released
registration remain B-005 work. No Truss runtime or production adoption gate is
introduced.

### Original mixed-home relationship qualification (2026-10-07)

Endpoint home selection is now exercised independently: Customer keys in props
and Orders keys in native rows, then Customer keys in native rows and Orders
keys in props. The public Configuration re-admits each selected original
property against its binding, and every direction/operation has the same logical
plan and result columns as the all-native reference. The fixture numeric renderer
selects the admitted access carrier per field instead of assuming one home for
an entire query; this is trusted Rust test composition, not a model-selected
executable procedure.

`tests/truss-postgresql/mixed-relationship-native.py` passes 72 PostgreSQL 17.9
cases across the two home arrangements and four public queries. Each valid
fixture executes every emitted prerequisite before SQL results; corrupt fixtures
execute prerequisites and suppress results. Cases cover empty and populated
owners, exact uint64 and above-JavaScript-safe key values, duplicate edge bags,
bounded lookahead, hidden overflow at either endpoint, duplicate/missing keys,
fractional unsigned keys, dangling endpoints and wrong endpoint types. Receipt:
[B-005-mixed-relationship-native.json](B-005-mixed-relationship-native.json).
No native Python/browser or production claim is added. Composite endpoint keys
and broader original scalar semantics remain open B-005 requirements.

### Independent composite endpoint admission (2026-10-07)

`relationship_definition::composite_tests::independent_composite_endpoint_keys_preserve_order_and_refuse_substitution`
authors three original model/binding cuts: source/target key arities 2/1, 1/2
and 2/3. Component order deliberately differs from record member order; the
cases include string, exact unsigned integer and decimal types. Source documents,
record definitions and key definitions are re-pinned from their actual authored
bytes before original Record and Relationship admission.

All six directed reads preserve the independently authored source and target key
identity order. Each produces exactly three typed catalog slots for physical
edge correlation, independent of logical key arity. For each read, substitutions
of either relationship endpoint's ordered storage-property list and either
resolved key are refused. Duplicate storage-property components refuse during
binding parsing; reversed lists refuse during original admission. This adds a
behavioral Rust regression, not native composite result or embedding evidence.
Public SQL lowering and PostgreSQL execution of composite relationship keys
remain open and must not be inferred from this admission result.

### Public original composite relationship SQL (2026-10-07)

`tests/truss-postgresql/build-composite-relationship-fixture.py` authors a new
original UMF document and physical binding with a two-component Customer key
`(part, id)` and three-component Orders key `(part, id, third)`. These keys use
required exact uint64 fields and an order different from declared member order.
The builder extends original owned composition metadata and re-pins every changed
nested artifact from actual bytes; it does not generate SQL or expected results.
Storage property IDs 20–22 avoid the existing layout's globally allocated IDs.

The conformance-only original loader is exercised directly by a Rust test,
`conformance_original::composite_tests::composite_relationships_compile_through_original_owned_configuration`.
It reconstructs the selected original definitions, then compiles four full public
requests through Configuration and Compiler: forward/inverse HAS_RELATED counts
and bounded RELATED_KEYS pages. Deterministic complete responses agree. Removing
any of the five endpoint component comparators refuses each request without SQL
or parameter artifacts; restoring registration restores the complete response.
The runtime's existing eight-cut allowlist is unchanged. No new Python/browser
or released registration claim follows from this Rust test.

`tests/truss-postgresql/composite-relationship-native.py` passes 32 PostgreSQL
17.9 cases against independently authored logical tuple expectations. Valid
fixtures include repeated individual key components with distinct complete keys,
exact uint64 maximum and above-JavaScript-safe values, lexicographic related-key
ordering, duplicate edge bags, truncation lookahead and empty results. Corrupt
fixtures include duplicate complete tuples at either endpoint, overflow and wrong
codec in an unconnected third component, missing key components and dangling
endpoints. Every emitted prerequisite is executed; invalid fixtures suppress
result SQL. Receipt:
[B-005-composite-relationship-native.json](B-005-composite-relationship-native.json).

This establishes composite native-row key behavior for the selected unsigned
numeric 2/3 subset. Composite mixed homes, heterogeneous key types, more original
scalar domains and full host transport qualification remain separate evidence
requirements. Synthetic fixtures do not establish installed Truss compatibility.

### Composite component home independence (2026-10-07)

The original composite relationship test now compiles four home arrangements:
all native rows, Customer's two components in props, Orders' three components in
props, and individual components split across homes (Customer.id and Orders.part
in props, with the remaining key components in rows). Each selected property is
re-admitted against its exact home definition. The conformance-only numeric
renderer reads the admitted per-field carrier for either location; its required
nonnullable uint64 domain is unchanged. Existing pinned runtime compositions and
allowlist remain unchanged.

All 16 public requests compile deterministically. Removing each of the five
required component comparators yields 80 blocked responses without SQL or
parameter artifacts; restoration returns the original response. The three new
arrangements emit twelve captured SQL artifacts.

`tests/truss-postgresql/composite-relationship-native.py --mixed` passes 96
PostgreSQL 17.9 cases using the same independent complete-tuple expectations as
the native-row baseline. The baseline's 32 cases also pass after the harness adds
per-component props storage. Wrong third-component codec tests retain native-row
codec corruption where applicable and use the wrong JSON kind when that
component resides in props. Missing/overflow/duplicate tuple/endpoint guards are
observed independently of query filters; corrupt cases do not execute result SQL.
Receipt: [B-005-composite-mixed-native.json](B-005-composite-mixed-native.json).

This is original Rust compiler and synthetic native PostgreSQL evidence for the
selected required uint64 composite subset, including within-key mixed homes.
Heterogeneous scalar key types, additional original scalar semantics, fresh
Python/browser transport and installed production compatibility are not claimed.

### Heterogeneous exact numeric relationship keys (2026-10-07)

`tests/truss-postgresql/build-heterogeneous-relationship-fixture.py` authors an
original model/composition variant whose Customer key is `(decimal(28,2),uint64)`
and Orders key is `(decimal(28,2),uint64,uint64)`. Changed source documents, leaf
codecs, value graphs, presence/home definitions and comparators are pinned from
their actual bytes. The exact decimal comparator uses finite numeric semantics,
forbids scale coercion and refuses nonfinite values. Expected SQL results are
independent of the fixture builder.

A separate Rust regression uses the same original owned configuration path for
all-native, Customer-props, Orders-props and within-key mixed homes. All sixteen
requests compile deterministically; each of the five required comparator removal
cases refuses each request without SQL/parameters, and restored registration
restores the full response. The test-only numeric renderer explicitly admits
required uint64 and decimal(28,2) operand domains. No broader decimal or runtime
binding allowlist claim is introduced.

`tests/truss-postgresql/composite-relationship-native.py --heterogeneous` passes
40 PostgreSQL 17.9 cases; adding `--mixed` passes 120 cases. The harness checks
exact decimal text, numeric lexicographic tuple ordering, repeated individual
components with unique complete tuples, edge bags, bound lookahead, empty results
and existing corruption guards. New controls put excess scale and precision
overflow into an unconnected decimal key component; emitted prerequisites reject
both and result SQL is suppressed. Receipts:
[B-005-heterogeneous-relationship-native.json](B-005-heterogeneous-relationship-native.json)
and [B-005-heterogeneous-mixed-native.json](B-005-heterogeneous-mixed-native.json).

This is original compiler and synthetic PostgreSQL evidence for the named exact
numeric heterogeneous subset. UTF-8/C-collation string components, other scalar
domains, broader source-token grammars and fresh Python/browser transports remain
unqualified. No installed Truss or production compatibility claim follows.

### Unicode string composite keys and explicit collation (2026-10-07)

`tests/truss-postgresql/build-string-relationship-fixture.py` authors an original
string/uint64 composite key variant with required Unicode string first components,
`unicode-text-C` comparators, UTF8 encoding and normalization `none`. String leaf
codecs preserve Unicode scalars and exclude numeric adoption artifacts. Changed
original artifacts are re-pinned from bytes; model content does not choose
executable procedures. The runtime's existing exact binding allowlist is unchanged.

The separate string Rust regression compiles sixteen requests across native-row,
Customer-props, Orders-props and within-key mixed homes, including eighty missing
component-comparator refusals and deterministic restoration. Its trusted
conformance renderer explicitly selects native `text_value` or the admitted
props carrier and applies `COLLATE pg_catalog."C"` to string fields/parameters.
The first PostgreSQL run exposed its previous native numeric-carrier assumption;
that test-only renderer was corrected and captures regenerated before qualification.

`tests/truss-postgresql/composite-relationship-native.py --string` passes 32
PostgreSQL 17.9 cases; adding `--mixed` passes 96. Composed `é` and decomposed
`e` plus combining acute remain distinct complete keys even with the same uint64
component, and ordering matches independent UTF8-byte/C expectations. Related-key
bags, complete-tuple uniqueness, empty results, lookahead and corruption checks
retain the earlier independent assertions.

Native text columns use an adversarial ICU `und-u-ks-level1` nondeterministic
collation. Every string case first positively verifies that this collation treats
the two Unicode spellings as equal. Emitted explicit C comparison/order then
preserves distinctness; this avoids inferring correctness from a database whose
default locale is already C. Receipts:
[B-005-string-relationship-native.json](B-005-string-relationship-native.json) and
[B-005-string-mixed-native.json](B-005-string-mixed-native.json).

All 138 core/PostgreSQL/runtime tests pass with `weft-runtime/test-original`.
This qualifies the selected original required Unicode-string/uint64 composite
subset over synthetic PostgreSQL fixtures. Other string facets/collations,
additional scalar domains, fresh Python/browser transport and installed Truss
compatibility remain outside this evidence.

### Original composite keyset continuation (2026-10-07)

The independent composite fixture builders now author two additional public
related-page requests: Customer continuation after `(part,id)` and Orders
continuation after `(part,id,third)`. Cursor values remain exact typed named
parameters. The related projections retain all independently admitted endpoint
keys; comparator-removal/refusal checks still cover all five required components.
Each uint64, decimal/uint64 and string/uint64 variant compiles 24 deterministic
requests over four home arrangements and 120 missing-comparator refusals.

The native matrix now passes 624 PostgreSQL 17.9 cases: uint64 48 native/144
mixed, decimal 60 native/180 mixed, and string 48 native/144 mixed. Of these,
208 exercise the new cursor requests. Independent expectations compare complete
logical tuples, rather than reusing emitted SQL. Controls include equal first
and second components with a greater third component, a greater earlier
component with a smaller later component, exact large integers, decimal text,
C-ordered Unicode, empty results and owner-wide corrupt-data rejection even
outside the cursor range. String cases retain the positive adversarial ICU
normalization-equivalence control. All emitted prerequisites precede result SQL;
corrupt cases suppress results.

All 138 core/PostgreSQL/runtime tests pass. Custody and HELIX artifact checks pass.
Existing composite/heterogeneous/string native receipts have been refreshed with
the expanded captures, input SQL hashes and harness hash. The original runtime
allowlist and production/embedding scope are unchanged. STP-003 now records
partial candidate execution evidence instead of leaving every criterion purely
planned; no story is closed by this checkpoint. Original optional/native-null
scalar roots, complete whole-entity composition and broader query/domain/host
qualification remain required B-005 work.

### Optional original scalar and complete entity projection (2026-10-07)

`tests/truss-postgresql/build-optional-scalar-fixture.py` authors a required
string/uint64 key plus an absent-allowed Unicode `note` Field. The Customer
Record.members order is deliberately `(note,id,part)`, while the key is
`(part,id)`. Source documents and every changed nested original definition are
re-pinned from actual bytes. All three selected members have independent original
property admissions; no field is inferred from a result alias or storage ID.

The failing whole-entity request found that OriginalBackend removed the
`project.entity` language capability even though its shared assembler already
consumes fully resolved ordered member projections. This capability is now
retained for original V02 composition. The new original Configuration regression
compiles whole-entity, explicit scalar and bounded whole-entity requests across
native-row and props homes for the optional member. It checks exact member/output
order, a `value` presence representation with `nativeNull:false`, deterministic
full responses and refusal without SQL when the optional property's original
admission is removed. Restoration restores the exact response.

`tests/truss-postgresql/optional-scalar-native.py` passes 48 PostgreSQL 17.9 cases.
Independent expectations distinguish an absent member, an empty Unicode string,
a Unicode value with trailing spaces and ordinary present values. The two-row
entity page still observes invalid data in later, non-returned owners. Corruption
controls cover explicit JSON null/null nodes, wrong value kind, SQL-null carrier,
missing root, duplicate state, orphan scalar, wrong codec, null-node payload and
non-object props roots as applicable to each home. Every emitted prerequisite is
executed before results; invalid cases suppress result SQL. Receipt:
[B-005-optional-scalar-native.json](B-005-optional-scalar-native.json).

The pinned presence definition expressly says native null is unsupported without
its separately selected profile. This checkpoint preserves that refusal and does
not invent null permission from optional availability or ideal scalar type.
Positive null-profile support, additional optional scalar domains, entities mixing
recursive roots and fresh Python/browser transports remain separate requirements.
This is original compiler/synthetic PostgreSQL evidence, not installed Truss
compatibility or a full B-005 acceptance claim.

### Original optional/entity native Python and browser boundary (2026-10-07)

The conformance-only runtime now selects two additional exact original binding
cuts: optional `note` in native rows and in JSONB props. The owned compositions
retain original codec/profile/inventory/selector bytes; props selection explicitly
has no row join. The loader preserves this distinction and re-admits each
configuration against the current Catalog and binding on every request. Captures
contain data only, never executable model procedures. The runtime remains a
static ten-cut conformance allowlist, not a released generic plugin protocol.

Six full public request/response pairs cover complete scalar entities, explicit
scalar projection and bounded entity pages in both homes. The runtime regression
matches their entire responses and deterministic repeats. All 140 core,
PostgreSQL and runtime tests pass with `weft-runtime/test-original`. Independent
custody checks now cover ten compositions and seventeen complete responses.

A fresh maturin 1.9.6 / Python 3.12.14 native wheel built with `test-original`
passes seventeen complete response cases, seventeen deterministic repeats and
fifty-one candidate-disabled/unsupported-cut/wrong-digest refusals. This is 68
corpus cases plus 17 repeat calls (85 native compile calls). Subprocess entry
points are disabled during compilation. The wheel is local macOS 11 ARM64 abi3
cp39; no other native platforms are newly qualified.

A fresh wasm32 build and wasm-bindgen 0.2.105 web glue pass the same 68 corpus
cases byte-for-byte against Python in Chromium 153.0.8010.12 / Playwright 1.62.1.
Network APIs are disabled after initialization, Node globals are absent, and
imports remain string/error/memory glue only. Existing compile wrapper transport
and trap-retirement checks also pass. WASM size is 14,147,313 bytes; observed
memory grows from 2,293,760 to 5,636,096 bytes. Browser execution requires macOS
process-services escalation, not compiler IO permission.

[Receipt](B-005-optional-embedding.json) pins source, full transport/composition
captures, logs, summaries and binaries. Wheel SHA256:
`8498c8c4ed89cf559da127ffd9ab1ca89a4f02269844c1b3e5a03bdefb636c0a`.
WASM SHA256:
`139cfc87a8bf7bf03c821937676d37a411045a0b6e573d755d6ce3b51a114b2e`.
Summaries are in `tests/truss-postgresql/evidence/optional-embedding/`.

Reproduction uses the existing isolated toolchain and `test-original` feature:
`cargo test -p weft-core -p weft-postgresql -p weft-runtime --features weft-runtime/test-original`,
`maturin build --manifest-path crates/weft-python/Cargo.toml --features test-original --interpreter /private/tmp/weft-toolchain/venv/bin/python`,
and `cargo build -p weft-wasm --features test-original --target wasm32-unknown-unknown`.
Install the fresh wheel into a new target directory, set PYTHONPATH to it and run
`tests/truss-postgresql/original-embedding-check.py`. Generate web glue with
wasm-bindgen and run `tests/compile/browser-check.mjs` using its WEFT_PROBE_JS,
WEFT_PROBE_WASM, WEFT_PROBE_API=compile_json and generated
`target/b005/original-embedding/{cases,reports}.json` inputs.

This adds actual embedding evidence for the selected original optional-scalar
and complete three-scalar entity subset; it does not add native database or
production compatibility claims. Native null remains unsupported by the pinned
presence profile. Entities mixing recursive roots, broader scalar/query domains
and complete host/story qualification remain B-005 work.

### Complete entities with scalar and recursive roots (2026-10-07)

`tests/truss-postgresql/build-recursive-entity-fixtures.py` authors seven model
cuts combining an original recursive member, optional Unicode note and required
string/uint64 key. The recursive closures are structured address, cyclic address,
map, nested sequence, numeric address, numeric map and string sequence. Original
Field/Record definitions for each selected closure are retained; owning Record
members and source document pins are re-authored explicitly. Declared member order
is `(recursive-member,note,id,part)` and key order remains `(part,id)`.

The new original Configuration regression compiles all 28 whole-entity requests:
seven graph shapes, optional note in native rows or JSONB props, and limits ten
or one. Recursive members use their independently admitted complete-value-tree
homes and trusted fixture procedures; scalar members use their independent homes.
Exact column/member order is checked. Removing the recursive property's admission
refuses without SQL, and restoring it restores the complete deterministic response.
No member is silently dropped or flattened during whole-entity expansion.

`tests/truss-postgresql/recursive-entity-native.py` passes 168 PostgreSQL 17.9
cases with independent scalar/presence values and the previously authored exact
logical recursive fixture expectations. The harness executes every emitted
prerequisite before permitting result SQL. Hidden corruption in the recursive
source bytes, key codec or optional note is detected even beyond the one-row page.
A corrupt tree belonging to an unrelated object discriminator is excluded by the
selected owner scope. Missing compound state yields absence for the three authored
optional address roots and refusal for the four required sequence/map roots.
Exact numeric recursive leaves remain strings, optional absence differs from an
empty string, and all outputs retain declared order. Receipt:
[B-005-recursive-entity-native.json](B-005-recursive-entity-native.json).

All 141 core/PostgreSQL/runtime tests pass. HELIX and ten-composition/seventeen-
response custody checks pass. This adds original Rust and synthetic native SQL
composition evidence; it does not add these new whole-entity cuts to the runtime
conformance allowlist or qualify fresh Python/browser execution for them. Native
null remains unsupported by the pinned profile. Broader scalar/query domains,
whole-entity embedding and full host/story qualification remain B-005 work.

### Complete recursive entity Python and browser embedding (2026-10-07)

The test-original registry now admits fourteen exact scalar/recursive entity
configurations: seven recursive shapes with each optional-note storage home.
Captured configurations retain original definition/source bytes and independent
property selectors. The custody check verifies all twenty-four configurations
against forty-five complete public responses, including the previous scalar,
compound and relationship cuts. Runtime tests compare every new response exactly
and repeat compilation deterministically. All 142 Rust tests pass.

Fresh CPython abi3 and browser WASM builds pass the expanded 180-case corpus:
forty-five full-response comparisons and 135 candidate/digest/binding refusals.
Python additionally repeats all forty-five successful calls, for 225 native
compiler calls with subprocess execution disabled. Real Chromium 153.0.8010.12
matches response bytes using the fresh WASM binary, without Node globals or
external network dependencies. Committed summaries and artifact hashes are in
[B-005-entity-embedding.json](B-005-entity-embedding.json).

This closes the embedding evidence gap for the selected complete recursive entity
cuts. Existing synthetic PostgreSQL evidence remains separately scoped. Native
null remains unsupported by the pinned profile; broader scalar/query domains,
entities with multiple recursive roots and complete host/story qualification
remain B-005 work. This allowlist is a conformance facility, not a released generic
backend registration or production compatibility claim.

### Host obligation orchestration subset (2026-10-07)

The test-only host executor in `host_obligation_fixture.py` consumes the emitted
original entity obligations. Its independent tests were run before the executor
existed (missing-module failure), then pass 532 cases across the twenty-eight
whole-entity transports. Unknown obligations or context meanings, missing required
context, mismatched guard parameters, changed binding/model/layout and incomplete
visibility refuse before any SQL callback. Violations, null guard results and
check errors prevent the data query. Data-query errors or authority/pin/visibility
changes at publication prevent buffered results from escaping. Successful paths
execute every preparation check before the query and recheck context afterward;
exact numeric text is returned unchanged.

Receipt: [B-005-host-obligation-orchestration.json](B-005-host-obligation-orchestration.json).
This is injected host callback evidence, not a native driver, real authorization,
transaction isolation, descriptor decoding or production host claim. The host
must supply transaction-affine trusted callbacks; database execution remains
outside the compiler. Native transport with these newer original paths and full
host/story qualification remain open.

### Original entity native driver and publication gates (2026-10-07)

`entity-driver-native.py` passes 252 executions through psycopg 3.2.10 and
PostgreSQL 17.9 using an owned ephemeral localhost-only fixture database.
The 168 independently authored native entity setups are reused without deriving
expected values from compiler output. Each generated prerequisite and result
statement uses its unchanged native `$n` placeholders and server preparation in
one host-owned transaction. The host sends explicit text parameter OIDs: implicit
unknown typing failed on unused slots in the shared preparation parameter vector.
Native guard int8/text carriers are checked and explicitly converted to the
obligation's exact count spelling. Result integer carriers are verified as OID 25
text; recursive JSON decoding rejects unqualified float tokens. Independent exact
outputs preserve absent/empty values, member order and recursive numeric strings.

The additional 84 executions deliberately change authority, binding or complete
visibility after data execution and before publication. Buffered results are
refused. Corrupt stored-data cases refuse before the result query. Context
callbacks are injected fixtures, not real authorization-system qualification.
The helper now rejects unknown obligation fields/guard meanings and unexpected
failure codes; its independent callback corpus passes 588 cases.

Receipt: [B-005-entity-driver-native.json](B-005-entity-driver-native.json).
This supplies actual native transport and orchestration evidence for the selected
original entity cuts. Production policy, general descriptor decoder adoption,
broader original scalar/query domains and full story qualification remain open.

### Unqualified native leaf refusal (2026-10-07)

The original conformance native leaf procedure previously panicked on an
unqualified scalar family or non-scalar shape. A tests-before-code regression
reproduced the panic. The procedure now returns a WFT-BINDING diagnostic for
boolean, decimal, timestamp, binary, unknown families and a sequence node instead
of unwinding. This does not qualify those domains: it preserves explicit refusal
while broader domain procedures remain unfinished. All 143 core/PostgreSQL/runtime
tests pass; custody and specification checks pass. Receipt:
[B-005-native-leaf-refusal.json](B-005-native-leaf-refusal.json).

### Signed and unsigned integer operand lowering (2026-10-07)

The comparator layer already admits exact signed/unsigned integer widths 1–64.
The original conformance expression procedure previously restricted all integer
operands to uint64, causing the new tests-before-code width regression to refuse.
It now accepts exactly the admitted width/signedness facet shape and preserves
those logical facets and exact literal tokens in emitted parameter slots.
Unknown top-level/nested facets, missing signedness, widths zero/65 and nullable
operands refuse without allocating slots. Decimal(28,2) and Unicode-C paths keep
their previous selected scope. No storage codec or comparator is inferred from a
literal or from this procedure: original binding/domain admission remains separate.

All 145 Rust tests pass. `integer-operand-native.py` passes 256 PostgreSQL 17.9
prepared executions for independently calculated minima/maxima at every signed
and unsigned width. Native numeric evaluation returns exact text, including
int64 minimum and uint64 maximum. Receipts:
[B-005-integer-operand-lowering.json](B-005-integer-operand-lowering.json) and
[B-005-integer-operand-native.json](B-005-integer-operand-native.json).
This qualifies operand rendering/transport, not full original signed property
storage, a new public runtime preset, fresh embedding or production compatibility.
Those broader paths remain required B-005 work.
