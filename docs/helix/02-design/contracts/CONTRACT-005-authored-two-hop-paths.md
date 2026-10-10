---
ddx:
  id: CONTRACT-005
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-001
      kind: informed_by
    - id: FEAT-005
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# CONTRACT-005: Authored two-hop paths and grouped counts

**Type:** language/library/host boundary. **Version proposal:** paired
`weft-sql/0.4.0`, `weft-ir/0.4.0`, `weft-compile/0.4.0` and
`weft-application-result/0.4.0`. **Status:** draft proposal for interface review;
syntax and version assignments below are not approved or a support claim.

## Purpose

Define bounded path reads and exact grouped path counts over two explicitly
authored UMF relationships. Realize CONTRACT-001's typed bag semantics and
CONTRACT-004's multiplicity and exact-count requirements without composing
truncated related-key lists or inferring relationships from property equality.

## Scope and Boundaries

The initial subset has exactly two monomorphic directed authored relationships,
each independently resolved forward or through its authored inverse name. Selected
lifecycle, endpoint, key, scalar and multiplicity semantics MUST be established
under the original owning UMF version. Association Records, polymorphic endpoints,
unknown selected meanings, arbitrary recursion and cross-backend traversal are
excluded. Both hops execute under one complete held publication context.
Weft owns language/IR; the registered backend owns physical occurrence carriers;
the host owns authorization, native checks, execution and result release.

## Normative Surface — proposed syntax

Within this draft, MUST statements specify the proposed interface for review.
Old dialects MUST continue refusing these forms.

```sql
SELECT l.id,
       RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 20)
         AS supplier_paths
FROM order_lines l ORDER BY l.id;

SELECT l.id, COUNT(*) AS path_count,
       COUNT_DISTINCT_PATH_TARGETS(p) AS supplier_count
FROM order_lines l
CROSS JOIN EXPAND_PATHS(l."order_lines.product_id", "products.supplier_id") AS p
GROUP BY l.id ORDER BY l.id;
```

`RELATED_PATHS(first, second, bound)` is a correlated collection projection.
This initial collection form is admitted only in nongrouped, nonaggregate queries.
A grouped/aggregate query MUST refuse it rather than choose a representative
starting occurrence; path expansion owns the grouped-count surface below.
`EXPAND_PATHS(first, second)` is a lateral bag source: each matching two-edge path
multiplies its current left input occurrence once. Its alias identifies the typed
path occurrence; it is not an inferred Record or a property-join alias. Initial
path alias usage is limited to `COUNT_DISTINCT_PATH_TARGETS(alias)`; key/edge
projections from expansion require a separately specified addition.

The first argument is a scan-qualified authored relationship name; the second
is one quoted or unquoted relationship name resolved from the first traversal's
terminal Record using CONTRACT-001 identifier rules. Neither argument is SQL text
or a parameter. Resolution MUST establish exact fully qualified Record continuity,
including owning document and revision, without synthesizing model elements.
Inverse traversal MUST preserve authored relationship identity and orientation.
Bound is a literal integer from 1 through 1,000, independent of outer LIMIT.

Initial expansion occurs once, after ordinary source joins and before WHERE,
grouping and projection. The first argument can name any visible scan occurrence.
Chained expansions, path-alias filters, LEFT path expansion and expansion mixed
with bounded path projection are excluded in this subset. Ordinary source filters,
joins, grouping and exact scalar outputs retain their selected dialect semantics.

An ordinary LEFT JOIN may make the selected starting scan relationally absent.
For a potentially unmatched starting scan, the collection descriptor MUST carry
`outerJoin: {scan, record}`, naming the exact scan occurrence and original Record
identity, and the plan MUST retain `outerJoinScans` as in the owning LEFT contract.
Its closed carrier is `{state:"absent"}` for an unmatched root, or
`{state:"value",value:{items:[...],truncated:boolean}}` for a present root.
A present root with no paths has a value containing an empty collection; it is
never absent. No `state:"null"` carrier is admitted for this collection itself.
A bare SQL NULL cell cannot establish relational absence.

Absence MUST use the genuine proven non-null physical row-identity sentinel
from the original pinned starting scan, under `join.left`,
`value.outerJoinPresence` and mandatory `outerJoin.matchIntegrity` obligations
from the owning LEFT contract. The new profile must explicitly admit their
composition with paths and counts; the historical LEFT profile is unchanged.
Missing/forged sentinel or scan/Record correspondence refuses. Nullable property
values, source missing values, empty collections and synthetic constants cannot
substitute for the sentinel. Source NULL keys remain invalid under the selected
key contract. For an absent root, `EXPAND_PATHS` produces no rows. Ordinary WHERE
and grouping retain their existing relational absence and three-valued semantics.

### Typed IR

The proposed closed representation introduces these owned shapes; exact JSON
schemas MUST be finalized and reviewed before implementation relies on them.

| Shape | Required members and meaning |
|---|---|
| PathRead | `startScan`: visible occurrence ID; `hops`: ordered array of exactly two existing RelationshipRead descriptors; `span`: original whole expression span; `hopSpans`: two original name spans |
| RelatedPaths | `op: relatedPaths`, `path`: PathRead, `bound`: positive integer |
| PathExpansion | `occurrence`: unique path alias occurrence, `path`: PathRead; attached to the relational stage before filters |
| CountDistinctPathTargets | `op: countDistinctPathTargets`, `pathOccurrence`: visible expansion occurrence; exact mathematical Integer output |

Each RelationshipRead retains its original identity, direction, fully qualified
from/to Record identities, authored source/target keys with ordered Field
identities/types, original multiplicities and lifecycle. ModelPin records retain
original document bytes, owning version, revision and digest. Backend physical
names, edge IDs and native scalar widths MUST NOT enter logical identities.

### Closed 0.4 wire selection

The initial request retains the 0.3 members with the exact
`weft-compile/0.4.0` / `weft-sql/0.4.0` pair. It has no `readProfile`
member; the plan retains `readProfile:null`. Existing 0.2 entity-page or
count-summary profiles do not select path operations.

The plan may contain one `pathExpansion` object, never an array:
`{"occurrence":"paths","path":PathRead}`. Omit this member when no expansion
is selected. Its stage is after all ordinary joins and before filters. An
occurrence must be unique among all scan and expansion occurrences. A count
expression is `{"op":"countDistinctPathTargets","pathOccurrence":"paths",
"type":{"family":"integer","facets":{},"nullable":false}}`; its type uses the existing mathematical Integer
shape, not a native-width annotation. This is the existing LogicalType shape for a nonnullable mathematical Integer.

A path column representation is the closed object
`{"kind":"relatedPaths","path":PathRead,"startRecord":Identity,"bound":2,
"edgeEncoding":"signed64-decimal/0.1"}`. Only a potentially unmatched start
adds `"outerJoin":{"scan":"root","record":Identity}`. PathRead and each hop
retain the existing ordered authored key descriptors, including component types.
The outerJoin scan and Record must equal the selected path start. Original column
position, outputName, optional admitted carrierName, sourceIdentities and nullable
metadata retain their established shapes. Every path column has nullable:false,
including tagged absence; SQL NULL is never a path carrier.

A distinct-target count representation retains the established exact Integer
scalar carrier/decoder (logicalType exactly {family:"integer",facets:{},
nullable:false}, carrier:"text", decoder:"exact-integer") and additionally
carries a closed `pathTarget` object:
`{"pathOccurrence":"paths","record":Identity,"key":AuthoredKey}`.
The Record is the second hop's terminal Record; its document/revision/module/
element identity and ordered authored key define distinctness within the held
source. The descriptor does not introduce a caller-chosen source label or local
edge token. The backend binding and publication obligations establish the exact
source-system/type/model population for that Record. The host must compare this
object to the referenced expansion; a schema-valid detached descriptor is not
admitted identity evidence.

Path results have their own closed `pathCollection` definition containing only
`items` and `truncated`; each item contains only `intermediate`, `terminal` and
`edges`, with exactly two String edges. All key atoms are Strings as in relatedKeys:
String unchanged, Boolean exactly "true" or "false", Integer/Decimal exact
lexical strings validated by their declared decoders. No native Boolean atom is
admitted. Do not apply uniqueItems: equal keys and
parallel edges retain their bag occurrences. A nullable-root representation
selects only `pathPresence`: `{state:"absent"}` or
`{state:"value",value:pathCollection}`. Neither `state:null`, bare null nor a
generic value object is accepted. Required-root representations select the
unwrapped pathCollection directly. No old generic presence fallback applies.

JSON Schema proves closed members, exact version/tag alternatives, two hops/two
spans/two edge tokens, positive bound at most 1000 and primitive carrier shapes.
Resolver/descriptor validation proves occurrence references, hop continuity,
original pin/key/type correspondence, capabilities and descriptor equality.
Held input/capacity/representation checks establish their declared constraints.
Reviewed backend correspondence and independent native conformance establish full
bag enumeration and truncation semantics; zero-check success alone is no universal
proof of emitted SQL.
Structural validity alone discharges none of these latter obligations.

The new typed backend admission uses `weft-backend/0.3.0` and a new
`backend-manifest-v0.3.schema.json`, selecting only the exact initial 0.4
language/IR pair and explicitly selected new profile. Existing backend 0.2
manifest schemas, typed routes and registrations remain unchanged. Collection
requires relationship.twoHopPaths and result.pathOccurrences; expansion requires
relationship.twoHopPaths and relationship.pathExpansion; distinct-target count
adds aggregate.pathTargetDistinctCount. Existing selected aggregate/group/count
and LEFT capabilities remain independently mandatory before binding callbacks.

### Held checks and closed obligation payloads

Reuse the existing publication, scalarIntegrity and relationshipIntegrity
obligations without changing their payload shapes. Emit relationshipIntegrity
checks for both consumed hops, preserving complete authored endpoint, orphan,
source/type/model, unique key/edge and distinct-neighbor constraints. Retain
`outerJoin.matchIntegrity` with its existing scans payload for every selected
LEFT scan; path encoding adds no alternative presence authority.

Introduce only two path-specific host obligations. Both use the established
Obligation envelope (id, parameters, owner:"host", failureCode). For
`ashlar.path.occurrenceIntegrity`, failureCode is WFT-BINDING; parameters are
exactly `{phase:"before-user-query",samePublicationRequired:true,
noPartialPublication:true,paths:[{path:PathRead,edgeEncoding:"signed64-decimal/0.1"}],
edgeSchemas:[{pathIndex:0,hop:0,relationship:RelationshipIdentity,
table:PublicationTable,identityColumn:"id",nativeType:"BIGINT"},
{pathIndex:0,hop:1,relationship:RelationshipIdentity,table:PublicationTable,
identityColumn:"id",nativeType:"BIGINT"}],
checks:[{pathIndex:0,kind:"edgeEncoding",sql:"…",failureCode:"WFT-BINDING"}],
success:"one exact STRING count equal to 0 per check"}`. paths is nonempty;
pathIndex references its position. Check kind is exactly edgeEncoding,
intermediateIdentity or collectionEncoding. The first covers original native
non-null/unique IDs and reversible token/order encoding; the second
covers complete intermediate identity continuity; the third covers selected
collection keys/encoding/prefix-marker proof. Each selected collection includes
its bound in its paths entry; expansion entries omit bound. Every paths entry is closed: required path and edgeEncoding, optional bound
1..1000 only for a collection. Inventory is all relatedPaths output expressions
in output order, each with its bound, followed by the sole expansion if present
without bound. Repeated equal output expressions retain their separate positions.
For every pathIndex require exactly one edgeEncoding check and one
intermediateIdentity check; a collection additionally requires exactly one
collectionEncoding check, while expansion forbids that kind. No other or duplicate
(pathIndex,kind) pair is admitted.

occurrenceIntegrity parameters additionally require edgeSchemas, in pathIndex
then hop order, exactly two entries per path:
`{pathIndex:0,hop:0,relationship:RelationshipIdentity,table:PublicationTable,
identityColumn:"id",nativeType:"BIGINT"}`. PublicationTable is the existing
complete pinned table descriptor, not an unpinned SQL alias. hop is 0 or 1;
relationship equals that hop's original identity. Table/column must equal its
admitted physical binding. The host obtains the complete native field schema
from that exact UUID/version/table under the same active publication hold,
including every field's name, type and nullable Boolean, before SQL checks.
The identity field must actually have BIGINT type. Nullable:true schema metadata
is permitted only with the complete-source non-null value check; nullable:false
metadata never replaces that check. Missing callback, partial/mismatched schema,
stale pin or wrong type refuses even for an empty source. SQL zero counts cannot
prove native schema. Retain these actual observations with the check results.
Schema validation cannot attest that a SQL check establishes these meanings.

For `ashlar.path.countCapacity`, failureCode is WFT-CAPABILITY; parameters are
exactly `{phase:"before-user-query",samePublicationRequired:true,
noPartialPublication:true,nativeRepresentation:"signed64",
checks:[{pathOccurrence:"paths",kind:"targetDistinct",sql:"…",
failureCode:"WFT-CAPABILITY"}],
success:"one exact STRING count equal to 0 per check"}`. Check kind is exactly
pathRows or targetDistinct. Require one pathRows check when COUNT(*) consumes
the expansion and one targetDistinct check when a distinct-target output is
selected; multiple outputs of the same count share that check. Forbid duplicate
(pathOccurrence,kind) pairs and checks for unselected counts/occurrences. These checks use the existing aggregate-candidates
method: complete post-join/post-expansion/post-WHERE bags, per group before
HAVING/ORDER/LIMIT. targetDistinct uses the declared terminal Record/key identity;
pathRows preserves all edge pairs and left-input duplicates. Ordinary aggregate
capacity checks retain their existing owning obligations; no arithmetic-specific
DECIMAL coefficient claim is made for path counts. Unknown obligation members,
kinds or unfulfilled checks refuse. All SQL uses the same emitted ordered slots;
no value interpolation or second publication context is permitted.

### Occurrence result and order

The collection descriptor names the starting Record and both ordered relationship
and endpoint/key descriptors. Its typed carrier is UTF-8 JSON:
`{items:[{intermediate:[keyValues],terminal:[keyValues],edges:[token1,token2]}],truncated:boolean}`.
A potentially unmatched starting scan uses the closed tagged carrier above;
other collections use the unwrapped carrier. An absent root is not an empty collection. Every key value follows its exact declared scalar decoder; integers/decimals are
lexical strings. Keys preserve declared component order and arity.

An edge token is an opaque String identifying one actual edge occurrence under
the descriptor's relationship and complete publication context. The backend MUST
specify its exact lossless native-ID-to-token encoding and total order, prove
uniqueness within that scope, and reject malformed/duplicate stored IDs. Tokens
are not UMF keys, portable cross-engine identities, or fabricated Edge Records.
The descriptor scopes each token to its original relationship and publication;
a host MUST NOT compare tokens across those scopes.

The initial local-commerce Databricks path profile selects the original native
BIGINT edge identities. Each token is the canonical signed base-ten decimal
String of that exact 64-bit integer: zero is `"0"`, negative values begin with
one minus sign, and no leading plus, leading zero or negative zero is admitted.
The token is an exact reversible encoding of the original stored identity, not
a new graph identity, hash, logical key or source-table rewrite. Token comparison
uses the decoded signed integer order; lowering orders the original BIGINT
column before carrier serialization, never its lexical String representation.
The decoder must verify canonical form and the exact signed 64-bit range.

Actual complete native schema/type and full-source non-null/uniqueness checks
must establish this profile before execution; schema nullable metadata alone is
not a value-level guarantee. Different native ID types, including STRING, require
an explicit separately reviewed token encoding/order profile. A source key token
cannot stand in for the native edge ID. Original table/profile identities and
native types remain unchanged. Native negative/zero/positive and range-boundary
cases must verify the encoding and order before profile/compiler admission.

Items are ordered by exact intermediate key tuple, then terminal key tuple, then
first and second edge tokens under the admitted token order. Distinct stored edge
pairs MUST remain distinct occurrences even when both key tuples are equal.
An empty collection is `items:[]` and `truncated:false`. Truncation is true exactly
when the complete correlated path bag exceeds bound. Retain the first bound
ordered items and obtain a same-context lookahead or equivalent exact proof;
outer LIMIT MUST NOT determine this marker.

### Expansion and exact counts

A path occurrence is a starting input occurrence plus an ordered pair of actual
edge occurrences whose intermediate object identity matches completely. Native
identity joins MUST preserve source-system scope, object type and model identity;
local ID equality alone is insufficient. Two first-hop edges and three second-hop
edges connecting the same objects yield six occurrences. Empty expansions yield
no rows; collection projection leaves the starting row with an empty collection.

`COUNT(*)` over expansion counts the resulting relational bag, including ordinary
join duplicates and parallel paths. `COUNT_DISTINCT_PATH_TARGETS(p)` counts distinct
terminal object identities in each current group, using complete source/Record
identity and exact authored key semantics. It MUST NOT deduplicate paths before
other aggregates or treat a token/key string as unscoped identity. Different
starting/intermediate occurrences reaching one terminal yield multiple paths but
one distinct terminal within a group. Global empty counts return one row with
zero; empty grouped input has no groups. Counts retain mathematical Integer and
exact-or-error native capacity obligations; never count a bounded collection.
Capacity checks cover the complete post-join, post-expansion, WHERE-filtered bag
for each group, before HAVING or outer LIMIT can hide a group. Global-empty
counts retain zero. Display collection bounds never reduce this capacity scope.

Relationship degree checks count distinct associated Record instances separately
on each authored side, per CONTRACT-004. They MUST NOT count parallel edges or
path occurrences and MUST NOT deduplicate the path result to enforce degree.

### Backend and host obligations

Declare separate capabilities `relationship.twoHopPaths`,
`relationship.pathExpansion`, `aggregate.pathTargetDistinctCount` and
`result.pathOccurrences`; select only those actually required. Existing scalar,
comparison, count, model-version and publication capabilities still apply.
Unsupported selected operations MUST refuse before binding callbacks.

Both relationship bindings MUST name original logical identities and complete
physical source/target/edge mappings. All selected endpoint/key/scalar integrity,
unique-key, edge-identity, relationship scope/type, orphan and distinct-neighbor
degree checks MUST cover the consumed pinned sources before user SQL. Count
capacity and path encoding checks MUST execute over the same held context.
The host MUST enforce all required obligations, authorize every consumed endpoint
and edge, preserve the complete publication vector through cleanup, and withhold
results on check, decoding, drift, reader-close or engine-cleanup failure.

## Precedence and Compatibility

This proposed extension changes no 0.1/0.2/0.3 grammar, plan, result schema or
historical compiler artifact. Exact new language/IR/interface pairs and backend
profile/version/realization MUST be selected explicitly; no syntax-driven fallback,
automatic upgrade, model relabel or host SQL repair is allowed. The backend trait
version changes only if its typed interface requires it, independently of dialect
versioning. Closed new schemas MUST reject unknown fields and invalid version
pairings; thin Rust/Python/browser entrypoints share the same core.

## Error Semantics

| Condition | Outcome | Recovery |
|---|---|---|
| Old dialect, unsupported construct or malformed syntax | Existing syntax/version diagnostic; no executable artifact | Select an admitted reviewed version or correct authored query |
| Missing/ambiguous name or endpoint continuity mismatch | WFT-NAME-MISSING, WFT-NAME-AMBIGUOUS or WFT-TYPE with original span | Supply exact original model or correct authored path |
| Unknown selected meaning or missing capability | Existing semantic/capability refusal; no partial SQL | Admit the exact meaning/profile with evidence |
| Missing/stale mapping, orphan or duplicate identity | WFT-BINDING; no released result | Repair source/binding through its owning boundary |
| Unproved count/order/encoding capacity | WFT-CAPABILITY; no released result | Select a proved domain/profile |
| Unknown obligation, policy denial, publication drift or cleanup failure | Host refusal under CONTRACT-003; no released result | Re-establish an authorized coherent context; never silently retry against newer data |

## Examples and Review Decisions

The commerce path above resolves the original order line L1 through product P1
to supplier S1. Both actual edge identities MUST be observed from stored data;
this symbolic example does not supply their values. A parallel-edge witness with
two L1-to-P1 edges and three P1-to-S1 edges has path count six and distinct supplier
count one; bound five retains five ordered items with `truncated:true`.

Review must decide the proposed function spellings, 0.4 family, opaque token
encoding/order per backend, and whether the initial restricted expansion alias
is an acceptable public foundation. No syntax/version decision is implied by
this draft. A backend encoding gap blocks its lane, not the typed frontend or
other independent engines. A bounded collection alone cannot satisfy grouped
path-count acceptance.

## Separate required-root related-key profile

Select backend `ashlar.databricks.paths-keys`, version
`0.4.0-paths-keys-candidate`, target `spark4-delta4-paths-keys-candidate`
explicitly. Its runtime feature is `ashlar-databricks-paths-keys` and executable
is `weft-paths-keys`. It retains the exact 0.4 language/IR/compile pair and
Backend03 interface, with the existing path profile's 47 capabilities plus
`relationship.boundedKeys`. The existing `ashlar.databricks.paths` registration,
profile, artifacts and refusal behavior remain independently selected.
Registration, schema validity and candidate status establish no native or
production support claim.

`RELATED_KEYS(root.relationship, bound)` uses the existing resolved
RelationshipRead and original complete target AuthoredKey. Its root must be a
required visible Record occurrence; every selected key component must be required
String with an admitted exact home. Forward and authored inverse traversal are
supported. Bound is 1..1000. Aggregate/group/HAVING use, a potentially unmatched
LEFT root, and mixing with path expansion refuse. Required-root one-hop and
two-hop collection outputs may coexist; each retains its output position.
This extension introduces no readProfile, page key, new grammar or inferred
relationship from a property join.

The emitted column is the existing closed `relatedKeys` representation from
`compile-response-v0.4.schema.json`, with exact original relationship/key/type
identity, bound and nullable:false. Select the inherited closed `relatedKeys`
carrier in `application-result-v0.2.schema.json`: exactly
`{items:[["key-component"]],truncated:false}`, with String atoms and complete
key arity. Do not use that schema's generic presence alternatives. The
`application-result-v0.4.schema.json` root remains path-only and unchanged;
representation-specific decoder dispatch must not relabel this carrier as
pathCollection or permit null/absence fallback.

Items retain one occurrence per admitted edge, including equal complete key
tuples. Order by complete target key components using the admitted exact String
order, then signed native edge ID. Empty input is `{items:[],truncated:false}`.
Items are exactly the first bound occurrences; truncated is true iff the full
bag exceeds bound in the same held publication. Authored participation degree
continues to count DISTINCT associated Record instances, not edges or tuples.
Reuse relationshipIntegrity unchanged for complete authorized endpoints, source,
model revision, key uniqueness, edge uniqueness/orphans, degree and lifecycle.

### Closed one-hop integrity and collection capacity obligations

Both new obligations use the existing envelope with owner:"host". All parameter
objects and inventory/check entries below are closed; missing, unknown, duplicate
or unselected members refuse. Output positions are the existing one-based column
positions, never labels. Repeated equal expressions retain separate entries.

`ashlar.relatedKeys.collectionIntegrity` has failureCode WFT-BINDING and exactly:

```json
{
  "phase":"before-user-query",
  "samePublicationRequired":true,
  "noPartialPublication":true,
  "collections":[{"outputPosition":1,"startScan":"root","relationship":"RelationshipRead","bound":2}],
  "edgeSchemas":[{"outputPosition":1,"relationship":"RelationshipIdentity","table":"PublicationTable","identityColumn":"id","nativeType":"BIGINT"}],
  "checks":[{"outputPosition":1,"kind":"collectionEncoding","sql":"emitted check SQL","failureCode":"WFT-BINDING"}],
  "success":"one exact STRING count equal to 0 per check"
}
```

The quoted type names in this example stand for their complete existing typed
objects. collections is exactly every selected RelatedKeys output in output
order; edgeSchemas has exactly one corresponding entry per collection, including
standalone edges not consumed by a two-hop path. RelationshipIdentity equals
RelationshipRead.identity; table/column/type equal the admitted immutable
NativeEdgeSource. For each output require exactly one collectionEncoding check
covering every consumed edge id IS NULL refusal and uniqueness of original edge
IDs across the complete authorized source, plus reversible complete String-key
carrier encoding and prefix/marker correspondence. A single NULL ID must refuse;
key-only carrier encoding and the reused relationshipIntegrity checks cannot
stand in for this value check. Check the full source even when outer selection
returns no rows or the bad edge lies outside the bounded prefix. Do not treat
that SQL check as native schema proof. Obtain the
complete native table field schema, ordered names/types/nullable Booleans, from
its exact UUID/version/table in the same hold before SQL. Require actual BIGINT
id and retain nullable:true only with full-source non-null/uniqueness guards.
Missing, partial, stale or inconsistent observations refuse even on empty tables.
Existing path occurrenceIntegrity remains mandatory for selected two-hop outputs.

`ashlar.relatedKeys.ordinalCapacity` has failureCode WFT-CAPABILITY and exactly:

```json
{
  "phase":"before-user-query",
  "samePublicationRequired":true,
  "noPartialPublication":true,
  "nativeRepresentation":"decimal38",
  "maximum":"99999999999999999999999999999999999999",
  "collections":[{"outputPosition":1,"kind":"relatedKeys"}],
  "checks":[{"outputPosition":1,"kind":"fullOccurrencePrefix","sql":"emitted check SQL","failureCode":"WFT-CAPABILITY"}],
  "success":"one exact STRING count equal to 0 per check"
}
```

Its inventory is all selected RelatedKeys and RelatedPaths collection outputs in
output order, with kind exactly relatedKeys or relatedPaths and exactly one
fullOccurrencePrefix check per output. Require it whenever any such collection
is selected in this profile. This capacity is separate from signed64 aggregate
pathRows/targetDistinct counts and their existing countCapacity obligations.

Use the existing two-hop wide-prefix algorithm for both collection forms:
`TRY_SUM(CAST(1 AS DECIMAL(38,0)))` over the complete per-owner occurrence bag,
ordered by the complete selected keys and native edge identities, with ROWS
BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW. Check NULL overflow across the full
population before outer filtering/ORDER/LIMIT or bound-plus-one lookahead can
hide it. Never narrow the internal ordinal or use ROW_NUMBER followed by a cast;
that cannot widen its native counter. A nonempty overflowing bag refuses;
empty input remains an empty collection. The maximum is a finite representation
bound, not a promised operational fanout. Encoding/prefix checks and independently
reviewed lowering/native bag conformance establish truncation; a zero guard
alone is not a proof of arbitrary SQL.

Host admission validates exact inventories against the typed plan, original
bindings and selected profile, then obtains all native schemas and runs owning
source/public-UMF checks before dependent capacity/encoding guards and user SQL.
Use the emitted parameter slots unchanged, one active publication reader and
ordinary protected ACK. Buffer bounded cells; validate carrier/key order and
bag multiplicity; close publication/resource checks, reader and engine before
report release. No profile fallback, SQL repair, source policy grant or transfer
of another realization's native evidence is permitted.
