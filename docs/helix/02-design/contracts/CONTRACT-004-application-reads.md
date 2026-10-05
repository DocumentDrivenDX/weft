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

**Versions:** proposed `weft-sql/0.2.0`, `weft-ir/0.2.0`, `weft-compile/0.2.0` and recognizer profile `weft-application-read/0.2.0`. **Status:** required product outcomes; draft concrete extension, not implemented support. The owner requested incorporation of [PR #2's discovery input](../../00-discover/application-read-requirements-input.md).

## Purpose and Scope

Support complete entity reads, bounded keyset pages, counts, related keys and typed source parameters. Keep CONTRACT-001's 0.1 behavior and fixtures unchanged. Version selection is explicit; no query upgrades itself based on target or recognizer acceptance. B-002A finalizes executable grammar/IR/result schemas and fixtures before plugin/adapter work relies on these constructs.

## Whole-entity projection and presence

`SELECT c.* FROM Customer c` selects every declared Field in the exact ordered `Record.members` list, resolving local dependencies by identity. Bare `*` stays excluded. Duplicate output labels block; explicit scalar projections may coexist only without collisions. Unknown/missing selected member meaning refuses rather than omitting the member.

Each output descriptor names field identity, cardinality, availability, exact type graph, carrier and decoder. Core `required` asserts a value; `absent-allowed` permits absence; `unspecified` asserts neither and cannot establish an application-read representation by itself. Explicit native null is a separate capability, not an inferred meaning of `absent-allowed`. A host-visible cell uses a tagged presence envelope `{state:"absent"}`, `{state:"null"}` or `{state:"value",value:...}` when those states are meaningful. A binding that cannot distinguish required states refuses. Plain SQL NULL cannot substitute for both absence and explicit null.

Lists have ordered items and a recursively declared item Field type, following `itemType`. Structured values follow explicit `references` with role `record-type`; the descriptor is an identity graph, with cycles represented as references rather than recursively expanded forever. Missing item/record meaning blocks. Exact numeric leaves use strings inside structured JSON carriers; no native JSON-number conversion through JS/Python floats. Map keys, item absence, explicit native-null permission and storage encoding require a recognized native/binding profile; core cardinality alone never chooses their encoding. An entire structured value is not silently flattened, truncated or JSON-stringified without a declared descriptor.

## Bounded deterministic reads and keyset continuation

Application entity-page profile requires `ORDER BY` the complete fields of an authored unique key, in its declared order and ASC, plus `LIMIT n`. `n` is a positive literal integer up to a declared profile bound; propose 1,000 as the initial application safety maximum, with compiler rejection above it. Backend ordering must prove exact numeric order or a stated exact text order/collation and selected key uniqueness/stability. No OFFSET or hidden row cut. General 0.1 queries remain governed by their own unbounded semantics.

Scalar `key_column > literal_or_parameter` supports single-field key continuation. Composite continuation uses `(key_col1,key_col2,...) > (value1,value2,...)` with lexicographic comparison in the same complete key order. A conjunction of independent `>` comparisons is not a composite keyset cursor. Tuple arity/type mismatch refuses. Cursor context includes model/binding revisions and publication/snapshot assumptions; concurrent mutations can change pages unless the host supplies a consistent read context. Never claim cross-page snapshot stability solely from ORDER BY.

## Counting

`COUNT(*)` counts rows in the current bag, including join duplicates. It returns an exact mathematical integer, with text carrier/decoder. Global empty input returns one row containing zero; grouped empty input returns zero rows. Count cannot infer distinct entity count or collapse relationships. Bounded native count domains and exact-or-error obligations are explicit.

## Relationships and inverse names

Proposed source forms: `HAS_RELATED(c.orders, KEY(:order_id))` as a WHERE predicate and `RELATED_KEYS(c.orders, 20) AS orders` as a projection. Parameter/literal tuples follow the target's explicitly named authored key, in declared component order. Resolve `orders` against supplied module-level authored `relationships`, including its stable ID and source endpoint; inverse names resolve from the target endpoint with direction reversed. Ambiguous names, unkeyed inverse endpoints, polymorphic endpoints without a complete supported domain, association records without supported traversal and unknown multiplicity/lifecycle semantics refuse. An ordinary property join is never converted into a relationship automatically.

HAS_RELATED is existential and cannot multiply the source row. RELATED_KEYS preserves endpoint/edge multiplicity as declared by the relationship/profile; its result is `{items:[keyTuples],truncated:boolean}`. Items are ordered by the complete target key; equal key tuples retain multiplicity. Truncation is true exactly when more than the requested bound exist in the same qualified read context. Empty related results have an empty list and false marker. Backend must obtain a lookahead or equivalent exact cardinality proof; silently cutting a list is forbidden. Multiple endpoint variants require tagged key tuples, not an inferred common identity.

These outcomes require known bindings of UMF relationship IDs to actual Truss edges/foreign keys or Ashlar layout. Missing backend access paths/capabilities refuse. Read authorization applies to source and related entities; inverse traversal cannot elevate privileges.

## Parameters, literals and recognizable subsets

Source marker `:name` is a typed parameter after model resolution. The 0.2 request supplies a named parameter map with explicit family/type and exact string value; references intersect all required field/key domains. Missing, surplus or incompatible parameter bindings refuse. The emitter allocates ordered native slots from typed data; source values never become SQL fragments. Null/presence parameter forms need explicit semantics and are not inferred from string `null`. Literal quote escaping, Unicode scalar preservation, base-ten exact numbers and no exponent/backslash-escape convention retain CONTRACT-001 rules with fixtures.

Publish named `entity-page`, `count-summary` and `related-entity-page` subsets of this dialect, including grammar, required model/key/type checks and positive/refusal fixtures. A host recognizer must state its profile/version, refuse excluded constructs by name and perform the subset's supplied-model checks before claiming a query is Weft-valid. Syntactic recognition alone is not semantic validation or a backend capability claim. Recognition never constructs target SQL from caller text. Replacing a recognizer with the shared compiler must preserve acceptance and meaning for the pinned subset; language expansion requires an explicit profile change.

## Backend and Host Obligations

Truss requires qualified type/key/property-home scans, exact missing/null/list decoding, bounded key order and authored edge mappings. Ashlar requires accepted gold tables/types, exact ordering/count behavior, published read context and matching related layouts. Hosts report the selected publication/feed position and enforce policy/prepared execution. Compiler output can declare these obligations; it cannot obtain the position or attest authorization itself.

## Compatibility, Diagnostics and Validation

B-002A must finalize profile maxima, grammar and JSON schemas; these concrete extension details remain draft until that executable slice. Backend interfaces must include capabilities for whole-entity member representations, presence, key order/tuple comparison, limit, count, relationships and parameters. Unsupported selected meaning, representation, ordering or obligations blocks atomically with a named construct. Independent fixtures cover all US-007 criteria, including composite keys, inverse names, absence/null/empty-list distinctions and unchanged 0.1 refusals. No native backend or release support is implied by this contract.
