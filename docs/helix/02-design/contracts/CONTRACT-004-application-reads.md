---
ddx:
  id: CONTRACT-004
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-005
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# CONTRACT-004: Versioned application reads

**Versions:** proposed `weft-sql/0.2.0`, `weft-ir/0.2.0`, `weft-compile/0.2.0` and recognizer profile `weft-application-read/0.2.0`. **Status:** required product outcomes; draft concrete extension with B-002A frontend evidence; public/native backend support remains unqualified. The owner requested incorporation of [PR #2's discovery input](../../00-discover/application-read-requirements-input.md).

## Purpose and Scope

Support complete entity reads, bounded keyset pages, counts, related keys and typed source parameters. Keep CONTRACT-001's 0.1 behavior and fixtures unchanged. Version selection is explicit; no query upgrades itself based on target or recognizer acceptance. B-002A finalizes executable grammar/IR/result schemas and fixtures before plugin/adapter work relies on these constructs.

## Whole-entity projection and presence

`SELECT c.* FROM Customer c` selects every declared Field in the exact ordered `Record.members` list, resolving local dependencies by identity. Bare `*` stays excluded. Duplicate output labels block; explicit scalar projections may coexist only without collisions. Unknown/missing selected member meaning refuses rather than omitting the member.

Each output descriptor names field identity, cardinality, availability, exact type graph, carrier and decoder. Core `required` asserts a value; `absent-allowed` permits absence; `unspecified` asserts neither and cannot establish an application-read representation by itself. Explicit native null is a separate capability, not an inferred meaning of `absent-allowed`. A host-visible cell uses a tagged presence envelope `{state:"absent"}`, `{state:"null"}` or `{state:"value",value:...}` when those states are meaningful. A binding that cannot distinguish required states refuses. Plain SQL NULL cannot substitute for both absence and explicit null.

Lists have ordered items and a recursively declared item Field type, following `itemType`. Structured values follow explicit `references` with role `record-type`; the descriptor is an identity graph, with cycles represented as references rather than recursively expanded forever. Missing item/record meaning blocks. Exact numeric leaves use strings inside structured JSON carriers; no native JSON-number conversion through JS/Python floats. Map keys, item absence, explicit native-null permission and storage encoding require a recognized native/binding profile; core cardinality alone never chooses their encoding. An entire structured value is not silently flattened, truncated or JSON-stringified without a declared descriptor.

## Bounded deterministic reads and keyset continuation

Application entity-page profile requires `ORDER BY` the complete fields of an authored unique key, in its declared order and ASC, plus `LIMIT n`. `n` is a positive literal integer up to a declared profile bound; 1,000 is the initial application safety maximum, with compiler rejection above it or for quoted/fractional/negative bounds. Backend ordering must prove exact numeric order or a stated exact text order/collation and selected key uniqueness/stability. No OFFSET or hidden row cut. General 0.1 queries remain governed by their own unbounded semantics.

Scalar `key_column > literal_or_parameter` supports single-field key continuation. Composite continuation uses `(key_col1,key_col2,...) > (value1,value2,...)` with lexicographic comparison in the same complete key order. A conjunction of independent `>` comparisons is not a composite keyset cursor. Tuple arity/type mismatch refuses. Cursor context includes model/binding revisions and publication/snapshot assumptions; concurrent mutations can change pages unless the host supplies a consistent read context. Never claim cross-page snapshot stability solely from ORDER BY.

## Counting

`COUNT(*)` counts rows in the current bag, including join duplicates. It returns an exact mathematical integer, with text carrier/decoder. Global empty input returns one row containing zero; grouped empty input returns zero rows. Count cannot infer distinct entity count or collapse relationships. Bounded native count domains and exact-or-error obligations are explicit.

## Relationships and inverse names

Proposed source forms: `HAS_RELATED(c.orders, KEY(:order_id))` as a WHERE predicate and `RELATED_KEYS(c.orders, 20) AS orders` as a projection. Parameter/literal tuples follow the target's explicitly named authored key, in declared component order. Resolve `orders` against supplied module-level authored `relationships`, including its stable ID and source endpoint; inverse names resolve from the target endpoint with direction reversed. Ambiguous names, unkeyed inverse endpoints, polymorphic endpoints without a complete supported domain, association records without supported traversal and unknown multiplicity/lifecycle semantics refuse. An ordinary property join is never converted into a relationship automatically.

HAS_RELATED is existential and cannot multiply the source row. RELATED_KEYS preserves endpoint/edge multiplicity as declared by the relationship/profile; its result is `{items:[keyTuples],truncated:boolean}`. Items are ordered by the complete target key; equal key tuples retain multiplicity. Truncation is true exactly when more than the requested bound exist in the same qualified read context. Empty related results have an empty list and false marker. Backend must obtain a lookahead or equivalent exact cardinality proof; silently cutting a list is forbidden. Multiple endpoint variants require tagged key tuples, not an inferred common identity.

### Participation degree and occurrence bags

For the supported owning UMF core 0.7/0.8 relationship meaning, participation
multiplicity MUST count distinct associated endpoint Record instances in the
complete authorized pinned dataset. It MUST NOT count edge occurrences, key
component values or projected key tuples. After verifying source scope, endpoint
Record type and identity, schema revision, endpoint existence and key uniqueness,
a backend MAY count distinct physical endpoint pairs within that exact relationship
scope. Forward target participation groups by source instance; inverse source
participation groups by target instance. Zero-associated instances MUST remain in
minimum-participation checks. Unknown endpoint meaning or incomplete authority
MUST refuse rather than infer a degree.

Independent edge occurrence identity and uniqueness remain separate constraints.
Two different edges between the same source and target contribute one neighbor to
each degree, but MUST remain two occurrences in RELATED_KEYS, exact lookahead and
truncation. Deduplicating the degree input MUST NOT deduplicate result inputs,
existential source rows, occurrence ordering or any future path bag. Distinct
physical endpoint pairs are a guarded representation of original Record identity,
not a replacement authored key or an invented association Record.

This clarification changes only relationship-integrity degree checks for newly
built compiler realizations. It does not change language, logical-plan, binding or
result shapes, nor silently upgrade an immutable compiler package or trusted
index. A previously qualified realization retains its original bytes and guard
receipts. Hosts MUST separately admit the exact new compiler realization and
its changed checks before execution; they MUST NOT label old occurrence-count
checks as distinct-neighbor evidence. Non-relationship artifacts and relationship
result SQL MUST retain their existing behavior. Native degree/parallel-occurrence
qualification is separate from compiler tests. Two-hop traversal remains outside
this application-read surface until separately versioned.

These outcomes require known bindings of UMF relationship IDs to actual Truss edges/foreign keys or Ashlar layout. Missing backend access paths/capabilities refuse. Read authorization applies to source and related entities; inverse traversal cannot elevate privileges.

## Parameters, literals and recognizable subsets

Source marker `:name` is a typed parameter after model resolution. The 0.2 request supplies a named parameter map with explicit family/type and exact string value; references intersect all required field/key domains. Missing, surplus or incompatible parameter bindings refuse. The emitter allocates ordered native slots from typed data; source values never become SQL fragments. Null/presence parameter forms need explicit semantics and are not inferred from string `null`. Literal quote escaping, Unicode scalar preservation, base-ten exact numbers and no exponent/backslash-escape convention retain CONTRACT-001 rules with fixtures.

Publish named `entity-page`, `count-summary` and `related-entity-page` subsets of this dialect, including grammar, required model/key/type checks and positive/refusal fixtures. A host recognizer must state its profile/version, refuse excluded constructs by name and perform the subset's supplied-model checks before claiming a query is Weft-valid. Syntactic recognition alone is not semantic validation or a backend capability claim. Recognition never constructs target SQL from caller text. Replacing a recognizer with the shared compiler must preserve acceptance and meaning for the pinned subset; language expansion requires an explicit profile change.

## Backend and Host Obligations

Truss requires qualified type/key/property-home scans, exact missing/null/list decoding, bounded key order and authored edge mappings. Ashlar requires accepted gold tables/types, exact ordering/count behavior, published read context and matching related layouts. Hosts report the selected publication/feed position and enforce policy/prepared execution. Compiler output can declare these obligations; it cannot obtain the position or attest authorization itself.

## Compatibility, Diagnostics and Validation

B-002A implements the draft frontend extension against the [0.2 grammar](weft-sql-v0.2.ebnf), [typed plan schema](logical-plan-v0.2.schema.json), [request schema](compile-request-v0.2.schema.json) and [value carrier schema](application-result-v0.2.schema.json). These structures and maxima have structural, independent result, resource/refusal and real-browser component evidence in [B-002A](../../04-build/evidence/B-002A-application-reads.md); public/native backend qualification remains incomplete. Backend interfaces must include capabilities for whole-entity member representations, presence, key order/tuple comparison, limit, count, relationships and parameters. Unsupported selected meaning, representation, ordering or obligations blocks atomically with a named construct. Independent fixtures cover all US-007 criteria, including composite keys, inverse names, absence/null/empty-list distinctions and unchanged 0.1 refusals. No native backend or release support is implied by this contract.


## Executable draft details for B-002A

The explicit 0.2 frontend retains 0.1 relational operations and adds COUNT(*),
whole-entity expansion, related reads, comparison, ordering, bounds and named
parameters. No version is inferred. The test frontend report remains distinct
from CONTRACT-003's public compile response; B-004 implements that boundary.
A named profile is supplied as `readProfile: {version:
"weft-application-read/0.2.0", subset: "entity-page"}` (or `count-summary` /
`related-entity-page`). Omitting it selects the general 0.2 dialect, which does
not imply boundedness or deterministic pages.

The typed IR is a sequence of relational stages: source and ordered inner joins,
conjunctive filters, aggregation when indicated, projection, order, then limit.
Joins preserve bag duplicates; HAS_RELATED is existential. Global aggregates
produce one row on empty input; grouped aggregates produce none. Ordering fields
remain available through the relational stages even when not projected. An
entity page identifies its selected authored `pageKey` as well as ordering
fields; a backend must qualify that identity's uniqueness/stability. All fields
in a tuple comparison retain their order; parameter domains are attached at each
use, with validation against every domain before a plan can be returned.

Named input bindings are `{family:"integer", value:"18446744073709551615"}` (or
boolean/string/decimal). Keys are ASCII identifiers and case-folded; duplicate
folded names refuse. Parameter markers are unquoted. Boolean values are exactly
`true` / `false`; exact numbers match `-?[0-9]+(\.[0-9]+)?`, without exponent,
plus sign, whitespace or implicit rounding. String values preserve Unicode and
quote characters and exclude NUL in this initial text profile. All values stay
typed data; they are never SQL fragments. Each occurrence can require a narrower
field domain than another occurrence of the same parameter.

`entity-page` excludes joins and aggregation, requires one source and a complete,
unambiguous authored key in ASC order with a positive LIMIT through 1,000.
Cursors compare that entire ordered key to a literal/parameter tuple.
`related-entity-page` has the same page constraints and requires a related
projection or predicate. `entity-page` excludes both. `count-summary` requires
COUNT(*) with optional grouped scalar projections and equality filters; inner
joins are permitted. It excludes SUM, entity/related projections, related filters
and cursor predicates. A global count excludes ORDER/LIMIT because it produces
one row; grouped count requires the complete grouping tuple in its declared order
and LIMIT. A grouping tuple is a unique result key even when it is not an entity
key. Backend text order must be qualified; exact logical text ordering in fixtures
uses Unicode scalar order, without normalization, trimming or locale folding.

The initial relationship traversal subset has one source, one target, directed
semantics, an explicitly named target key, known owned/independent lifecycle and
no association Record. Other selected variants refuse explicitly. Source key selection (and inverse target
key selection) uses a sole primary key, otherwise a sole authored key; ambiguity
refuses. Descriptors retain both multiplicities and lifecycle in their authored
orientation plus the inverse flag. Backend capability assessment must retain
these obligations; descriptors alone do not prove storage enforcement.

The descriptor graph is bounded to 4,096 reachable identities and depth 128 per
selected root, with cycles referring to prior identities. Availability is separate
from scalar type: a scalar descriptor's `nullable:false` describes its ideal
value domain and cannot authorize dropping absent/null distinctions. Ordered lists,
map string keys, structured members and exact numeric leaves need qualified
carriers/decoders. Native null permission must be explicit in the eventual binding
and output descriptor. The result schema models allowed envelope shapes, not
permission to use every state for every field.

## Separately versioned two-hop proposal

[CONTRACT-005](CONTRACT-005-authored-two-hop-paths.md) proposes bounded path
collections and relational path expansion/counting. This preserves the one-hop
surface here: RELATED_KEYS still returns its original terminal-key bag. A future
path bag must retain both actual edge occurrences and the intermediate identity;
degree remains distinct-associated-instance counting, separate from path bags.

## Projection-only exact tokens: compile/IR/backend 0.2.1

The explicit `weft-compile/0.2.1` envelope retains SQL dialect
`weft-sql/0.2.0` and selects `weft-ir/0.2.1`. It adds an identity descriptor
`{kind:"exactScalar", identity:..., availability:"required"|"absent-allowed",
scalarType:"timestamp"|"binary"}`. These are projection-only values, not new
members of the comparable `LogicalType.family` set. Original cardinality must
be one, references/item meaning must be absent and known facets must be empty.
Unknown selected semantics continue to refuse.

Required values MUST preserve the original timestamp text or binary token in a
JSON string. Optional values MUST retain the existing absent/null/value envelope.
A whole-entity selection MUST include every original member, including these
values, or refuse. The backend owns exact native carrier validation; core scalar
metadata does not choose a binary encoding or timestamp arithmetic semantics.
Comparison, order, grouping, authored key admission, parameters and aggregates
MUST NOT reinterpret these descriptors as comparable strings or numbers.

The versioned request, logical-plan, response and backend-manifest declarations
are the `*-v0.2.1.schema.json` files alongside this contract. New manifests use
`weft-backend/0.2.1` and explicitly declare SQL0.2.0/IR0.2.1. Old backend0.2.0
manifests and schemas stay closed to their original language pairs. A host MUST
check the returned transport, dialect, IR and selected backend versions before
executing or decoding. There is no automatic upgrade of an old request/backend.

The Truss runtime backend0.4.4 maps these descriptors to its existing validated
JSON-home text cells on runtime0.31/PostgreSQL18.6. Truss returns its existing
`TextValue` Python carrier. This scoped native path does not qualify timestamp
arithmetic, binary comparison, other database profiles or new browser artifacts.

## Relationship equality shorthand — 2026-10-13

A qualified single-column WHERE equality `s.addresses = 'key'` (or a named parameter) resolves to existing HAS_RELATED semantics when the actual Record has no authored member with that name and its direction-specific relationship target key has one component. The inverse name resolves from its actual target endpoint. Literal/parameter admission, exact key type and existential multiplicity follow HAS_RELATED unchanged. An authored member retains field semantics, including its existing refusal rules. Composite keys require explicit KEY tuples; ON, greater/tuple comparisons, bare names and field operands gain no shorthand meaning.

This is additive application-frontend support through the shared resolver, including existing application backend versions that already implement the resulting IR. It changes no transport or logical-plan schema and does not imply new backend capabilities or deployment qualification.

## Canonical Truss date queries — explicit 0.2.7 extension

The explicit `weft-compile/0.2.7`, `weft-ir/0.2.7` and
`weft-backend/0.2.7` contracts add the exact scalar descriptor
`{"kind":"exactScalar","scalarType":"date"}`. Truss backend `0.4.41`
extends backend `0.4.39` and requires the exact logical-query ABI `1.1` pin.
It retains the binding `0.5` shape and PostgreSQL18.6 target profile. Older
request, plan, backend and ABI selections MUST retain their original domains.
A model or target MUST NOT implicitly upgrade a source request.

This backend subset is canonical Gregorian `YYYY-MM-DD`, years `0001` through
`9999`, with valid month/day and leap-day combinations. It is an explicit Truss
value domain; UMF core's declaration of the `date` family alone does not establish
all temporal value or refinement semantics. Selected unsupported refinements
MUST refuse. Original date tokens MUST survive projection unchanged.

Equality admits a date Field against another date Field, a quoted canonical date
literal, or an exact parameter such as
`"day":{"scalarType":"date","value":"2000-02-29"}` in `exactParameters`.
The ordinary `parameters` string family MUST NOT substitute for a date.
The backend MUST compare canonical date tokens with C-collated text equality;
it MUST NOT cast them to timestamps, depend on session timezone or accept a
noncanonical spelling. An absent or present-null date operand yields SQL UNKNOWN
and does not match a WHERE or INNER JOIN equality. Projection MUST preserve the
existing absent/null/value distinctions. The new capabilities are `type.date`,
`compare.date.equal` and `compare.date.optionalEqual`.

For example, `SELECT a.day FROM A a WHERE a.day=:day` with the exact parameter
above returns the original date token for matching rows. An impossible or
noncanonical operand, including `1900-02-29` or `2024-2-01`, MUST block with
`WFT-DATE-DOMAIN` and no SQL. Date ordering, ranges, key/page-key components and
implicit date/string coercions are outside this extension and MUST NOT be inferred
from equality support. Hosts MUST validate exact native date slots and retain
original catalog, authority, buffered-source and final-handoff checks.


## Declared native table custody candidate — 2026-10-19

The explicit PostgreSQL18/Lakebase18 `table-lifecycle/0.1` verification
profile accepts original core0.10 documents with selected module-scope
`umf.tables`0.1.0 content. UMF CONTRACT-081 owns that extension's meaning;
Weft retains original source bytes and verifies their SHA-256, full owning-module
table/column mappings, primary Keys and declared relationship/FK correspondence.
An explicit `binding.nativeLifecycle` contains version
`umf.table-lifecycle/0.1`, the owning module and configuration/deployment digests.
These digests are requested custody context, never proof of installed authority.
The profile cannot mix legacy ColumnSet or catalog-fence admission.

Selected table declarations require this profile. Unknown selected extensions,
opaque nonempty command inventories, incomplete bindings and unsupported carrier
meaning refuse. Unselected module content remains retained without forcing its
interpretation. The current carrier subset is string, integer, decimal, boolean,
UUID and bits128. Date/timestamp, table mutations, search and composed related-read
admission remain unqualified; this profile must not silently route those paths
through legacy admission.

Compiled programs use the distinct `weft.table-read/0.1` version. Their
`obligations.tableLifecycle` retains source/configuration/deployment custody,
selected command policy pointers and required original UMF interpretation,
approved forward ledger/source-derived native witness, shared generation lock,
verified person/batch identity, current transitive roles, forced RLS and effective
privilege closure. Cursor scope additionally requires ledger head, principal and
role-scope digest. Every such program declares `nativeExecutionQualified:false`.
No database is inspected or authorized by compilation. Hosts must refuse this
program until their version-specific native admission and execution are qualified;
legacy raw-read catalog versions do not authorize it. This candidate introduces
no raw SQL execution endpoint and does not enable UMF's guarded client generator.

UMF's preservation schema intentionally accepts unknown object members. The
selected Weft interpretation derives a private closed schema for declared object
shapes, retaining explicit map-value schemas and the exact original source/schema
bytes. Unknown members in selected payload/table/column/type/reference/policy
objects refuse before lowering; arbitrary declared CoreLiteral map keys retain
their authored meaning. This closure is not contextual policy validation.

The pinned original schema and provenance live in
`spec/upstream/umf-tables-0.1.0.*`. Structural schema checks do not substitute for
UMF's contextual source interpretation or native policy/constraint enforcement.
Positive/refusal compiler evidence lives in
[the candidate receipt](../../04-build/evidence/praxis-table-lifecycle/README.md).

## Named native operation invocation candidate — 2026-10-19

The dedicated `weft.table-operation/0.1` invocation selects `module` and
`operation` from the exact original binding source and supplies typed CoreLiteral
inputs. Its closed request contains `version`, `target`, `binding`, `module`,
`operation`, `call`, and optional `idempotencyKey`. A call is either
`{kind:"single",inputs,after?,size?}` or `{kind:"batch",items}`. `after` is an ordered
original Key tuple for a declared page only. A batch invokes one declared write,
is bounded by its declaration, preserves input order and is one transaction.
Module identity must match the native lifecycle context. Reads reject replay
keys; writes enforce the declared required/optional caller key, bounded to256
UTF-8 bytes. Connection identity and role selection never come from this wire.

The shared Rust frontend admits original mappings and selected operation meaning,
checks exact input inventory/domains/native capacities and produces an opaque
admitted operation. Unknown versions/members, undeclared operations, invalid
cursor tuples, mutable-Key assignments and unsupported carriers refuse. Existing
raw read/action/search entry points do not acquire a command-inventory bypass.
The initial native lowering subset is string/integer/decimal/boolean/UUID/bits128;
date/time/binary and broader statement forms remain separately required gates.

PostgreSQL programs bind all input and literal data; source values never become
SQL fragments. Explicit predicates, ordered projections and original conflict
Keys drive lowering. Count uses an exact integer text result; scalar numerics
return exact text. Cardinality checks use lookahead. Page continuation adds the
complete ASC Key tuple comparison in the same native ordering. Mutation programs
return an affected-row marker even without business projections. A declared
maximum is an admission/transaction outcome: excess effects must roll back the
entire operation/batch, never be clipped by a hidden write LIMIT. RETURNING and
ON CONFLICT add their actual SELECT privilege/policy requirements.

Compilation authenticates no native deployment or person/batch role. Programs
retain original request/source, declaration and invocation digests, complete
transaction/replay/cursor obligations and `nativeExecutionQualified:false`.
Full source recompilation checks a program; a self-rehashed digest alone is no
proof. Native ledger/current generation, effective privileges, forced RLS,
policy/principal scope, atomic durable receipts and host transaction execution
remain mandatory before admitting a generated operation. No release or full
Praxis support follows from compiler acceptance alone.


The `pg18-effect-count/0.1` write guard counts direct target rows through a
materialized data-modifying CTE and raises SQLSTATE22012 on excess effects. The
statement fails before a result can commit; it never clips a write. Source-native
foreign-key actions retain their own declared PostgreSQL semantics and are not
included in this target-row count. This is not an all-graph effects bound or a
claim that native referential actions apply child-table RLS as direct DML does.
Host batch atomicity and durable outcomes still require their own execution gate.

`eq`/`ne` are null-safe; `in` and boolean composition preserve ordinary SQL
three-valued membership semantics, including NULL. Prefix data is literal data,
including percent/underscore. Known integer controls normalize to typed plan
integers while the complete original source/request retains authored lexical
bytes. Cursor metadata requires source/principal binding; it does not attest
that an untrusted `after` tuple is already authenticated. DBAPI transport always
binds a parameter tuple, including the empty tuple for percent escaping.


Single `many`/`page` calls may lower their requested `size` to1 through the
source-declared maximum; write/singleton/count and batch calls cannot override
it. Result metadata distinguishes effective and declared maxima. Canonical
`nativeInputs`, `nativeAfter` and literal `nativeValue` carry validated driver
scalars. Integer and decimal tokens normalize exactly, including extreme-exponent
zero, without passing floats or unbounded numeric lexemes to PostgreSQL. Original
tokens remain in the retained request/source. Driver binding expansion is bounded
to8MiB; emitted programs remain bounded to32MiB and the ordinary closed JSON
reader's depth/node limits. Drivers bind these issued scalars, not reparsed tokens.

Read results carry `nativeReadGuard:{profile:"pg18-read-frame/0.1",
maxNativeBytes:8388608,sqlstate:"22012",includesLookahead:true,partialFrames:false}`.
The backend materializes the bounded read and measures uncompressed visible
value octets with `octet_length`, including lookahead, before returning a frame.
An oversized frame raises SQLSTATE22012; a TOAST pointer or compressed datum
size cannot substitute for logical octets. The outer projection retains exactly
the declared output aliases. Hidden native sort projections preserve original
ordering, including fields omitted from the result, without sorting scalar text
coercions or exposing the hidden values. Count participates in the same guard.
The SDK additionally checks cardinality, exact carriers and serialized wire size;
this compiler guard alone grants no execution or principal authority.

Mutation results additionally carry `nativeWriteFrameGuard` with profile
`pg18-write-frame/0.1`, the same8388608-byte uncompressed-value limit,
SQLSTATE22012, `includesLookahead:false` and `partialFrames:false`. The existing
materialized effect-count guard also sums returning value octets and the effect
marker. An oversized RETURNING frame fails the native statement, rolling back
its effects. Empty business projections retain only the effect marker for host
counting; hosts return no invented empty business rows. Atomic batch execution
also checks its cumulative serialized wire budget before commit. Durable outcome
storage, authentication and recovery remain host/SDK obligations, not permissions
conferred by this compiler program.
