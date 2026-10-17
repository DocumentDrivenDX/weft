# Discovery input: Weft requirements from the UMF typed data access SDK

Proposed 2026-10-15 by UMF and amended the same day (WR-A10 to WR-A14, WR-B4 validation and edits, WR-B7 schema-neutral statements), for the owner's review and for Weft's governed
planning. This is source material, not an approved requirements document, and it
changes no existing Weft document. Weft owns the design and implementation of
everything below; UMF owns the semantics in "Shared semantics" and will not build
a competing lowering for any form Weft delivers. The UMF source of this input is
`umf/docs/helix/04-build/weft-requirements-typed-access.md` on UMF branch
`codex/typed-persistence-specs`.

## How to use this document

Work tranche by tranche, in the priority order at the end. Each requirement has an
ID (`WR-A1` and so on) for traceability in Weft's own HELIX artifacts. Where this
document proposes syntax (the grammar delta), Weft may choose different syntax as
long as the semantics and the compile outputs are preserved; tell UMF so its DSL
renderer matches. Anything here that conflicts with Weft's existing contracts
should be raised, not silently resolved.

## Context

UMF is building a schema-first Python data access SDK:

1. A UMF generator emits typed Python models and typed column expressions from UMF
   core 0.8 documents.
2. Developers declare queries and bulk commands in a typed Python DSL. A build step
   renders each declaration to **Weft SQL** and calls Weft's Python binding to
   compile it for every selected target. The compiled outputs are checked into the
   application as a catalog.
3. At runtime only catalog entries execute; caller values are bound parameters.
   Runtime adapters run the native statements on the host's own connection (raw
   tables) or through Truss and Ashlar.

So Weft is called **at build time**, and its outputs must be complete enough to
execute later without Weft, except for bulk mutation plans until WR-I5 lands.
UMF specifications: CONTRACT-071 (DSL and catalog), CONTRACT-072 (runtime
targets), CONTRACT-073 (native SQL action profile), CONTRACT-074 (search), in
`umf/docs/helix/02-design/contracts/`.

Weft today admits `=` and `>` predicates joined by `AND`, keyset pages ordered by
the complete unique key ascending, `LIMIT` up to 1,000, counts with equality
filters, `SUM`, equality joins, `HAS_RELATED` and `RELATED_KEYS`, with backends for
Truss on PostgreSQL and Ashlar on Databricks. The requirements below extend that.

## Interface requirements (all tranches)

- **WR-I1 Build-time binding.** A released, versioned Python distribution exposing
  the compile entry points used below, deterministic for identical inputs, with
  no network access during compilation. UMF pins exact versions.
- **WR-I2 Inputs.** Weft SQL text; supplied core 0.8 modules with pins; target
  `{backendId, backendVersion, targetProfile, bindingJson, bindingSha256}`;
  typed parameter declarations (WR-I4); the security input (WR-I6).
- **WR-I3 Outputs per compiled statement.**
  - native statement text;
  - an ordered parameter list with names and carriers (WR-I4);
  - an ordered output-column list with field identity, carrier and decoder;
  - host obligations (session settings, search path, isolation);
  - a **structural digest** of the statement with parameter values removed;
  - an echo of the logical Weft SQL and request identity;
  - deterministic diagnostics, or a `blocked` status with reasons.

  For backends whose page size or continuation are API arguments (the Truss page
  entry point), the output states that and carries the artifact to call.
- **WR-I4 Parameter and result carriers.** In addition to boolean, integer, exact
  decimal (text) and string, add:
  - date (ISO `YYYY-MM-DD`);
  - instant (signed epoch microseconds as text, UTC);
  - local timestamp (ISO without offset, microsecond precision);
  - binary;
  - **list parameters**: homogeneous lists of one scalar carrier, at most 1,000
    items, for WR-A4;
  - float32 vectors, for tranche C.

  Every carrier has one canonical text form; refuse non-canonical input rather
  than normalizing it.
- **WR-I5 Mutation plan templates.** Compile a bulk UPDATE once with typed
  parameter placeholders and later instantiate the plan from parameter values
  without recompiling. The instantiated plan must have the template's structural
  digest. Until this exists UMF recompiles at runtime and compares digests.
- **WR-I6 Security input for raw tables.** Compile request 0.5 requires `security`.
  Provide an explicit profile for raw-table targets meaning "native privileges
  only; no Weft row policy". It is recorded in the compile output and is not a
  default: requests must name it.

## Shared semantics (decided by UMF; apply identically on every backend)

- **S1 Value states.** A field value is present, explicit null, or missing.
  Missing exists only on property stores (Truss); relational columns have no
  missing state.
- **S2 Three-valued logic.** A comparison with a null or missing operand is
  UNKNOWN. `AND`, `OR` and `NOT` follow SQL three-valued logic. `WHERE` keeps only
  TRUE rows. This matches SQLAlchemy, Django, PySpark and Polars user expectations.
  For example, `NOT (status = 'open')` excludes rows where status is null.
- **S3 Null and missing tests.** `IS NULL` is TRUE for explicit null and for
  missing. `IS MISSING` is TRUE only for missing (property stores only; refuse
  elsewhere). Their `NOT` forms are the complements, and both tests are always
  TRUE or FALSE.
- **S4 Comparisons.**
  - Integers and decimals compare by exact mathematical value.
  - Strings compare by Unicode scalar value, in binary order (C collation), with
    no case or normalization folding.
  - Booleans support `=` and `<>` only.
  - Dates compare by calendar date.
  - Instants compare by UTC instant.
  - Local timestamps compare by wall-clock value.
  - Comparing an instant with a local timestamp refuses.
- **S5 Ordering.**
  - Each sort key is `ASC` or `DESC` with explicit null placement. The default is
    `NULLS LAST` for both directions, and lowering must emit it explicitly on
    every backend (Postgres and Spark defaults differ).
  - Missing sorts with null.
  - String order follows S4.
- **S6 Determinism.** Every ordered read ends with the Record's complete unique
  Key, ascending, appended after the requested sort keys unless it is already a
  prefix. Ties therefore have one order.
- **S7 Keyset continuation.**
  - A page's continuation is the tuple of the last row's values for every sort key,
    including the appended Key.
  - The next page holds the rows strictly after that tuple in the full ordering,
    with per-column direction and null placement. This is lexicographic order,
    not a single tuple `>`.
  - Continuations are API arguments or typed parameters, never text.
- **S8 Membership.**
  - `x IN (list)` is TRUE if any element equals `x`, UNKNOWN if `x` is null or
    missing, FALSE otherwise; an empty list gives FALSE.
  - Duplicates in the list are allowed.
  - `NOT IN` is the S2 negation.
- **S9 Ranges.** `x BETWEEN a AND b` means `x >= a AND x <= b`.
- **S10 Bounds.** Unchanged unless stated: `LIMIT` and page size at most 1,000.
- **S11 Aggregates.**
  - Aggregates ignore null and missing inputs. Over no non-null inputs, `SUM`,
    `MIN`, `MAX` and `AVG` return null and `COUNT` returns 0.
  - `SUM` is exact: integers sum to an exact integer (text carrier when outside
    signed 64-bit), decimals to an exact decimal whose scale is the input scale.
  - `MIN` and `MAX` follow S4 ordering.
  - `AVG` takes an explicit result scale and rounds the exact quotient half-even to
    that scale. A request without a scale refuses.
  - `COUNT(DISTINCT x)` is exact. An approximate distinct count is a separate
    function whose output is labeled approximate, with the backend's algorithm and
    documented error bound in the output descriptor.
- **S12 Buckets.**
  - A fixed-width bucket of a numeric value `x` with width `w` is
    `floor(x / w) * w`, computed exactly.
  - A date bucket truncates to `day`, ISO `week` (starting Monday), `month`,
    `quarter` or `year`.
  - Dates and local timestamps bucket by their own calendar values. Instants bucket
    only with an explicit IANA time zone parameter and are converted to that zone
    first.
- **S13 Prefix match.**
  - `STARTS_WITH(x, p)` is TRUE when the Unicode scalar sequence of `x` begins with
    that of `p`. It is binary, case-sensitive and does no normalization (S4).
  - An empty prefix matches every non-null value; a null or missing `x` is UNKNOWN.
  - Lowering must not interpret `%`, `_` or escape characters in `p`.
- **S14 Term match.**
  - `MATCHES_ALL_TERMS(x, q)` and `MATCHES_ANY_TERM(x, q)` tokenize `x` and `q` with
    the analyzer of a declared keyword index on `x` (UMF CONTRACT-074).
  - They are TRUE when every term (or any term) of `q` occurs in `x`. They are
    unranked boolean filters, and `q` is plain terms with no operators.
  - Without a declared keyword index on the target, they refuse.
- **S15 Snapshot pages.**
  - A snapshot page reads every page of one continuation at the same table versions,
    so no row is skipped or repeated because of concurrent writes.
  - The continuation records those versions.
  - Pages without a snapshot read live data. Keyset continuation then never repeats
    a row, but rows inserted or changed between pages may be missed or seen.
- **S16 Null-safe distinctness.** `x IS DISTINCT FROM v` is TRUE when exactly one
  side is null or missing, or both are present and unequal; FALSE otherwise. It is
  never UNKNOWN. `IS NOT DISTINCT FROM` is its complement. On property stores,
  missing and null are not distinct from each other.
- **S17 Relationship joins.**
  - A join follows one declared UMF relationship from a Record to a to-one target
    (each source row has at most one target row). It never multiplies source rows.
  - A required relationship is an inner join. An optional one is a left join whose
    target columns are null when there is no target.
  - Joins are only along declared relationships, one hop per join, at most four
    joins per query; arbitrary join conditions are not offered.

## Tranche A: query forms

Apply to the Truss runtime backend and the Ashlar backends where the stated
semantics can be met exactly; otherwise refuse that form on that backend with a
specific diagnostic. Tranche B backends must support all of tranche A.

- **WR-A1 Comparisons:** `<`, `<=`, `>=`, `<>` and `BETWEEN` on integer, decimal and
  string fields, and on date, instant and local timestamp fields (with WR-A8).
- **WR-A2 Null tests:** `IS [NOT] NULL` on every field type; `IS [NOT] MISSING` on
  property-store backends.
- **WR-A3 Boolean structure:** `OR`, `NOT` and parentheses, with S2 semantics, up
  to the existing predicate node and nesting bounds.
- **WR-A4 Membership:** `IN` and `NOT IN` with a list parameter (WR-I4) or a literal
  list.
- **WR-A5 Ordering:** `ORDER BY` any scalar field, with `ASC`/`DESC` and `NULLS
  FIRST`/`LAST`, for non-paged reads with `LIMIT`. S6 applies.
- **WR-A6 General keyset pages:** entity and related-entity pages ordered by any
  sort keys under S5 to S7, extending the current key-only ascending page profile.
- **WR-A7 Counts and existence:** `COUNT(*)` and grouped counts over any tranche-A
  predicate, plus an existence form (`LIMIT 1` probe or `EXISTS`) whose output is a
  boolean.
- **WR-A8 Temporal parameters:**
  - Date, instant and local timestamp parameters and comparisons.
  - On Truss, instants stored as RFC 3339 text with the author's offset must
    compare by UTC instant. If that cannot be lowered exactly, refuse on Truss and
    say so.
- **WR-A10 Aggregates and buckets:**
  - `SUM`, `MIN`, `MAX`, `AVG` (with scale), `COUNT`, `COUNT(DISTINCT)` and an
    approximate distinct count, under S11.
  - `GROUP BY` exact values, fixed-width buckets and date buckets (S12), up to 1,000
    groups.
  - `ORDER BY` any group key or aggregate output, with the complete group key
    appended as the tiebreak (the analogue of S6).
- **WR-A11 Prefix and term filters:** `STARTS_WITH` (S13) on string fields;
  `MATCHES_ALL_TERMS` and `MATCHES_ANY_TERM` (S14) on fields covered by a declared
  keyword index.
- **WR-A12 Snapshot pages (S15):**
  - On Delta and Unity Catalog, pin each page of a snapshot continuation with
    `VERSION AS OF`.
  - Expose the pinned versions in the continuation output.
  - Ashlar pages are already pinned to a publication.
  - On Postgres and Lakebase, refuse snapshot pages (only live pages there) unless
    Weft proposes an exact mechanism.
- **WR-A13 Null-safe distinctness:** `IS [NOT] DISTINCT FROM` (S16) for every
  comparable field type.
- **WR-A14 Relationship joins (S17):**
  - Joins along declared to-one relationships, inner or left as S17 requires,
    with predicates, projections and ordering over columns of joined Records.
  - Builds on Weft's existing equality joins and left-join contract. Output
    descriptors identify each column's Record and Field.
  - Refuse to-many joins; use `HAS_RELATED`, `RELATED_KEYS` or related pages for
    those.
- **WR-A9 Bulk UPDATE extensions (CONTRACT-009):**
  - `SET field = NULL` where the Field admits null.
  - `SET` values for integer, decimal, date, instant and local timestamp fields.
  - `WHERE` with any tranche-A predicate.
  - UMF will extend its plan contract (CONTRACT-902) with the matching null and
    value domains in parallel.

### Proposed grammar delta (Weft SQL 0.3)

```ebnf
predicate   = disjunction ;
disjunction = conjunction, { "OR", conjunction } ;
conjunction = negation, { "AND", negation } ;
negation    = [ "NOT" ], primary ;
primary     = "(", predicate, ")"
            | column, compare_op, value
            | column, "IS", [ "NOT" ], ( "NULL" | "MISSING" )
            | column, [ "NOT" ], "IN", "(", ( parameter | value, { ",", value } ), ")"
            | column, [ "NOT" ], "BETWEEN", value, "AND", value
            | column, "IS", [ "NOT" ], "DISTINCT", "FROM", value
            | "STARTS_WITH", "(", column, ",", value, ")"
            | ( "MATCHES_ALL_TERMS" | "MATCHES_ANY_TERM" ), "(", column, ",", value, ")"
            | "HAS_RELATED", "(", column, ",", "KEY", "(", key_value, { ",", key_value }, ")", ")" ;
compare_op  = "=" | "<>" | "<" | "<=" | ">" | ">=" ;
order       = "ORDER", "BY", order_item, { ",", order_item } ;
order_item  = ( column | aggregate | bucket ), [ "ASC" | "DESC" ], [ "NULLS", ( "FIRST" | "LAST" ) ] ;
aggregate   = ( "SUM" | "MIN" | "MAX" ), "(", column, ")"
            | "AVG_SCALE", "(", column, ",", nonnegative_integer_literal, ")"
            | "COUNT", "(", ( "*" | "DISTINCT", column ), ")"
            | "APPROX_COUNT_DISTINCT", "(", column, ")" ;
bucket      = "BUCKET_WIDTH", "(", column, ",", value, ")"
            | "DATE_BUCKET", "(", date_unit, ",", column, [ ",", "ZONE", value ], ")" ;
date_unit   = "DAY" | "WEEK" | "MONTH" | "QUARTER" | "YEAR" ;
group_item  = column | bucket ;
(* Projections may also be aggregate or bucket expressions with AS names;
   GROUP BY takes group_item, { ",", group_item }. *)
update_set  = field, "=", ( value | "NULL" ) ;
```

The additive raw-table DSL compiler spells explicit-scale half-even AVG as
`AVG_SCALE(column, scale_literal)` with a bounded literal scale in 0..1000.
This routine grammar resolution retains S11 and CONTRACT-071 `avg_(c, scale=n)`;
ordinary reviewed SQL `AVG(column)` keeps PostgreSQL semantics and does not
qualify DSL AVG. Native qualification remains separate from source admission.

Keyset continuation stays outside the grammar (S7): page profiles take the
continuation tuple as API arguments or typed parameters.

## Tranche B: raw-table backends

- **WR-B1 Backends and bindings.**
  - New backends for raw relational tables:
    - Postgres 16, 17 and 18;
    - Lakebase, a Postgres-compatible profile whose minor versions are applied
      automatically, so compile once per major version;
    - Unity Catalog through Databricks SQL warehouses and Databricks Runtime;
    - local Spark 4.x with Delta.
  - Bindings come from UMF CONTRACT-042 physical bindings plus the `umf.access`
    extension's ColumnSets (CONTRACT-070), which give exact native column types.
    Missing or incompatible column facts refuse.
- **WR-B7 Schema-neutral statements on Postgres and Lakebase.**
  - A binding may leave table schemas unqualified, meaning the host binds the
    schema once per connection through the role's `search_path`.
  - Lowering then emits unqualified table and type names, and qualifies every
    system object with `pg_catalog`, for example `pg_catalog.pg_has_role`, so
    `search_path` cannot redirect it.
  - The compile output records a host obligation: `search_path` resolves each
    table to the intended schema and keeps `pg_catalog` first (its default).
  - One compiled catalog then serves every deployment of an app, each with its
    own schema name.
- **WR-B2 Reads on Postgres and Lakebase.** All of tranche A, with output decoders
  for every WR-I4 carrier.
- **WR-B3 Set-based mutations on Postgres and Lakebase.**
  - CONTRACT-009 bulk UPDATE lowered to one native statement plus a preflight.
  - Bounded by `maxItems`: a selection larger than `maxItems` refuses with no
    effect.
  - Uses the mutation plan template (WR-I5).
  - No CONTRACT-903 manifest or CONTRACT-904 batching on raw tables.
- **WR-B4 Action lowering on Postgres and Lakebase (CONTRACT-073).** Lower a UMF
  action into a statement program meeting CONTRACT-073:
  - Covered: `graph-write/1` recipe effects (`create`, `set`, `delete`, `link`,
    `unlink`), explicit-key selectors, and CONTRACT-901 `rules/1` preconditions
    and postconditions.
  - The program runs in one `SERIALIZABLE` transaction.
  - A second, validate-only program evaluates parameter checks and every
    precondition (not only the first false one), reports all failing codes, and
    always rolls back.
  - Preconditions are evaluated before effects, and the first false one returns
    its failure code.
  - Role-membership checks come from the target's pinned role mapping.
  - The program captures `pg_current_xact_id()` for the receipt.
  - Output identifies the effects and the precondition codes, and the program
    returns the Keys of created and changed entities.
- **WR-B5 Reads on Unity Catalog and Spark/Delta.**
  - All of tranche A, with explicit `NULLS FIRST`/`LAST`.
  - Instants are read through epoch-microsecond text, never through Python
    `datetime` conversion.
- **WR-B6 Mutations and actions on Unity Catalog and Spark/Delta.**
  - Single-statement programs (`UPDATE`, `MERGE`) on both.
  - On Unity Catalog, multi-statement programs as `BEGIN ATOMIC ... END` on
    catalog-managed tables, with preconditions raised through `SIGNAL`.
  - Creates use `MERGE ... WHEN NOT MATCHED` on the Key.
  - Report whether committed table versions can be obtained for receipts. If they
    cannot, say so; UMF will leave Unity Catalog actions unqualified.

## Tranche C: search (CONTRACT-074)

- **WR-C1** Vector search on Postgres and Lakebase:
  - pgvector operators `<=>` (cosine), `<#>` (dot) and `<->` (l2);
  - `lakebase_vector` where declared;
  - exact or approximate per the authored index.
- **WR-C2** BM25 keyword search on Lakebase through `lakebase_text`. On Postgres,
  built-in full-text search only for `implementation-defined` ranking.
- **WR-C3** Unity Catalog `vector_search()` for vector indexes, including hybrid
  mode when the query asks for target-defined fusion.
- **WR-C4** Filters before search with "top k among matching rows" semantics, or an
  explicit refusal when a mechanism cannot guarantee it. Outputs carry key, rank
  and a labeled score.
- **WR-C5** Query text and vectors are bound parameters only; keyword syntax is
  plain terms (no operators interpreted).

## Acceptance corpus

UMF and Weft share one corpus, kept in Weft (proposed path:
`fixtures/umf-typed-access/`). Each case has:
- a core 0.8 model with `umf.access`;
- a binding;
- the Weft SQL that UMF's DSL renders;
- a small fixed dataset;
- the expected ordered rows, or the expected refusal.

The same expected rows apply to every backend that accepts the case. UMF will
contribute the first cases with tranche A. Minimum coverage:

- Every S2 truth-table cell for `AND`, `OR` and `NOT` with null and missing
  operands.
- `<>`, `NOT IN` and `NOT BETWEEN` over rows with nulls, which must exclude them.
- Empty `IN` lists.
- `DESC NULLS LAST` versus `DESC NULLS FIRST` on the same data, on Postgres and
  Spark.
- Keyset pages with mixed directions and nulls in the sort key, verifying that no
  row is skipped or repeated across pages.
- Instant comparisons across rows written with different offsets, including Truss.
- Decimal comparisons where `1.2`, `1.20` and `1.200` are equal.
- String ordering with non-ASCII characters (binary order).
- Bulk `SET NULL` refused for a non-nullable Field.
- Bulk selection larger than `maxItems` refused with no effect.
- Aggregates over all-null groups and empty inputs; `AVG` with scale rounding at a
  half-even tie; `SUM` of decimals keeping input scale; date buckets for instants in
  two time zones across a DST change.
- `STARTS_WITH` with a prefix containing `%`, `_` and a backslash.
- Snapshot pages on Delta with a concurrent write between pages: no skipped or
  repeated rows.
- `IS DISTINCT FROM` over every combination of null, missing and present values.
- A left join along an optional relationship with and without a target row; a
  refused to-many join.
- The same schema-neutral statement run under two roles whose `search_path`
  names different schemas, returning each schema's rows; a table shadowing
  attempt does not redirect `pg_catalog` functions.

## Priority

1. WR-I6, WR-I3 and WR-I4: the inputs and outputs UMF needs to call Weft at all.
2. WR-A1, WR-A2, WR-A3, WR-A5, WR-A8 and WR-A13, plus WR-B1, WR-B2 and WR-B7:
   comparisons, nulls, OR/NOT, null-safe distinctness, ordering and temporal values
   on raw Postgres and Lakebase, with schema-neutral statements. This is the first
   use-case path; the first consumer (an apps host on Lakebase, Postgres 18) needs
   it.
3. WR-A4, WR-A6, WR-A7, WR-A10 and WR-A11: membership, general keyset pages,
   counts and existence, aggregates and buckets, prefix and term filters, and
   WR-A14 relationship joins.
4. WR-I5, WR-A9 and WR-B3: bulk updates on raw tables.
5. WR-B4: actions on Postgres and Lakebase.
6. Tranche A on the Truss and Ashlar backends.
7. WR-B5 and WR-B6: Unity Catalog and Spark, with WR-A12 snapshot pages.
8. WR-I1 release.
9. Tranche C.

## Not asked of Weft

Connections, sessions, credentials and execution; runtime retries; authentication
or human attribution; schema migrations; embedding computation; search index
creation; the Python DSL and its type checking (UMF); action declaration meaning
(UMF CONTRACT-900/901).

## Prior art

Palantir's Ontology SDK offers similar typed filters, aggregations (sum, avg, min,
max, exact and approximate distinct; exact, fixed-width, range and date-bucket
groupings), a `$snapshot` paging option, and action application returning
validation results and edits. WR-A10 to WR-A12 and the WR-B4 additions close the
same gaps, with exactness rules stated explicitly.

## Questions for Weft

1. Can Truss instants be compared by UTC instant exactly (WR-A8), or should Truss
   refuse temporal ordering for now?
2. Is the continuation better as page API arguments everywhere (as the Truss page
   entry point does) or as typed parameters in the statement?
3. Which compile entry point should raw-table backends use: an extension of
   compile request 0.5 or a new request version?
4. For WR-B4, should action lowering be a new Weft entry point or an extension of
   the mutation compiler?
