---
ddx:
  id: weft.typed-sdk-raw-collections
  type: contract
  activity: design
  status: draft
  links:
    - id: CONTRACT-004
      kind: informed_by
    - id: weft.typed-sdk-raw-terms
      kind: informed_by
  authoring:
    home: repo
---

# Raw SDK membership, navigation and bounded related Keys

This is a PostgreSQL18/Lakebase18 compiler candidate. It does not qualify a
native deployment, change engine graph authority, or release temporal support.
The source model, binding, original requests, output metadata and all admission
programs remain covered by the original program digest.

## List membership

An original raw Parameter can declare `container:"list"`, `elementNullable`,
`maxItems` from1 through1000 and `nullable:false`. Its existing scalar `carrier`
is the element carrier. Scalar parameters omit all three list facets. The host
must validate and own the complete bounded list before binding it; no truncation
is permitted. The initial list profile refuses other targets and every use except
the sole operand of `IN (:parameter)` or `NOT IN (:parameter)`.

S8 membership is false for an empty list, unknown for a null left operand and a
nonempty list, true if any nonnull element equals the left operand, and false
otherwise. Null elements do not turn an unmatched nonnull value into unknown.
NOT IN applies S2 Boolean negation. The compiler emits this CASE/EXISTS program;
native SQL's nullable IN-list behavior is not substituted. Integer and Decimal
values remain exact numeric parameters and string equality uses binary C
collation. Literal lists follow the same null rule. Empty literal membership is spelled
`IN_EMPTY(field)` (or its Boolean negation); the owner resolves and type-checks
the original field before emitting FALSE. Its retained semanticFields inventory
is distinct from actual SQL-used columns after constant folding. The native
PostgreSQL parser dialect is not widened to accept IN().

## Original relationship operations

`HAS_RELATED(alias.name,KEY(...))` is an existential predicate.
`RELATED_KEYS(alias.name,n)` is a bounded projection for1 through1000 Keys.
The explicit inverse forms are
`HAS_INVERSE_RELATED(field,KEY(...),moduleLiteral,idLiteral)` and
`INVERSE_RELATED_KEYS(field,n,moduleLiteral,idLiteral)`. They retain canonical
identity and direction even for self relations or multiple incoming names.
The name resolves uniquely against the original authored relationship and its
owner endpoint, or its authored inverse name. The initial raw storage profile is
a declared directed monomorphic to-one FK mapping, with an inverse set view.
Association/independent edge storage is not inferred from property equality.
Forward tuples use the exact authored relationship target Key. Inverse tuples
use the sole primary Key, otherwise the sole authored Key; ambiguity refuses.
The selected paging Key never replaces the relationship Key.

The related Keys cell is JSON text with `{items:[keyTuples],truncated:boolean}`.
Its local output carrier is `related-keys`; decoder profile is
`pg-related-keys-json/0.1`, retaining bound, original relationship
module/id/direction, original Key identity and ordered Field/component carriers.
Numeric components are exact strings and Boolean components are JSON booleans.
Key components are nonnull. The compiler orders by the complete Key and retains
at most n+1 observations to establish exact truncation. It does not deduplicate
occurrences. Hosts strictly decode the original descriptor into typed Keys and
RelatedKeys/WithKeys wrappers before releasing results.

Filtered `weft.related-read/0.1` navigation retains the whole original source
and target request and relation module/id/direction. Its source entity set has
no implicit row cap; authored ORDER/LIMIT remain in the original subquery.
Canonical root `alias.*` expands inside the owner AST using the original ordered
Record members. Bare, foreign and modified wildcards refuse. Source aggregate
and grouped identities remain unqualified. Target selection/paging keeps its
separate selected Key, and all source admission/isolation obligations propagate.

## Native integrity and shape obligations

`nativeRelationships` version `umf.native-relationships/0.1` contains original
pre/post Boolean nativeAdmission statements and source/target physical ordered
pair metadata. The FK must be validated, enforced, nondeferrable, non-temporal,
with compatible null matching, native equality operators and active original
enforcement triggers. Parent/child inheritance topology refuses; relkind alone
does not prove that a table scan is a single admitted physical relation. Required forward participation requires NOT NULL source
components. Inverse maximum1 requires an actual immediate native unique index
over source FK components; nullable absent tuples remain permitted.

Other finite/minimum inverse degrees require the original distinct-source-Key
neighbor count, including zero-neighbor target rows. This candidate admits that
proof only with actual native row_security_active false for both tables, native
read-only repeatable-read/serializable transaction state, and the original
`readIsolation:"repeatable-read"` obligation. A caller completeness label is not
proof. An active-RLS completeness profile remains separately unqualified.

The source-bound nativeProjection admission also verifies the exact complete
physical column-name inventory, including Binary/Vector fields, in addition to
per-column types/facets/nullability/generation. Added or missing native columns
must refuse even for an empty table. Hosts execute all original pre/post programs
unchanged and withhold buffered rows on any failed or changed obligation.

## Qualification

Compiler source tests and installed-wheel controls are precursor evidence only.
Real native missing/wrong FK, added-column, degree/RLS/isolation, list null/empty,
alternate-Key, exact numeric, default wildcard, pagination and truncation controls
must qualify one immutable source/wheel/runtime cohort before release claims.
Temporal qualification remains held.
