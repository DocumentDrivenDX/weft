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
