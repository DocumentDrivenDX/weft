# Discovery input: Weft requirements from the UMF typed data access SDK

Proposed 2026-10-15 by UMF, for the owner's review and for Weft's governed
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
            | "HAS_RELATED", "(", column, ",", "KEY", "(", key_value, { ",", key_value }, ")", ")" ;
compare_op  = "=" | "<>" | "<" | "<=" | ">" | ">=" ;
order       = "ORDER", "BY", order_item, { ",", order_item } ;
order_item  = column, [ "ASC" | "DESC" ], [ "NULLS", ( "FIRST" | "LAST" ) ] ;
update_set  = field, "=", ( value | "NULL" ) ;
```

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
  - Preconditions are evaluated before effects, and the first false one returns
    its failure code.
  - Role-membership checks come from the target's pinned role mapping.
  - The program captures `pg_current_xact_id()` for the receipt.
  - Output identifies the effects and the precondition codes.
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

## Priority

1. WR-I6, WR-I3 and WR-I4: the inputs and outputs UMF needs to call Weft at all.
2. WR-A1, WR-A2, WR-A3, WR-A5 and WR-A8, plus WR-B1 and WR-B2: comparisons, nulls,
   OR/NOT, ordering and temporal values on raw Postgres and Lakebase. This is the
   first use-case path.
3. WR-A4, WR-A6 and WR-A7: membership, general keyset pages, counts and existence.
4. WR-I5, WR-A9 and WR-B3: bulk updates on raw tables.
5. WR-B4: actions on Postgres and Lakebase.
6. Tranche A on the Truss and Ashlar backends.
7. WR-B5 and WR-B6: Unity Catalog and Spark.
8. WR-I1 release.
9. Tranche C.

## Not asked of Weft

Connections, sessions, credentials and execution; runtime retries; authentication
or human attribution; schema migrations; embedding computation; search index
creation; the Python DSL and its type checking (UMF); action declaration meaning
(UMF CONTRACT-900/901).

## Questions for Weft

1. Can Truss instants be compared by UTC instant exactly (WR-A8), or should Truss
   refuse temporal ordering for now?
2. Is the continuation better as page API arguments everywhere (as the Truss page
   entry point does) or as typed parameters in the statement?
3. Which compile entry point should raw-table backends use: an extension of
   compile request 0.5 or a new request version?
4. For WR-B4, should action lowering be a new Weft entry point or an extension of
   the mutation compiler?
