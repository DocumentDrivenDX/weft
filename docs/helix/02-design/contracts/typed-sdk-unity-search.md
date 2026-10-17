---
ddx:
  id: weft.typed-sdk-unity-search
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
---

# Source-bound Unity Catalog candidate search

**Contract ID:** weft.typed-sdk-unity-search. **Type:** compiler/host boundary.
**Version:** `weft.unity-search/0.1`. **Status:** implementation candidate; native qualification pending.

## Purpose

Lower the original CONTRACT-074 non-temporal asynchronous candidate search and
bounded current-row hydration into a distinct owned program. This does not
reinterpret the PostgreSQL search program or Spark publication snapshots.
Compilation grants no native permission.

## Scope and Boundaries

The initial managed profile is a host-computed, authored approximate L2 vector
index bound to a unique original `unity-catalog` target, with asynchronous
freshness. An optional original keyword leg selects native HYBRID; standalone
keyword, BM25, RRF, target embedding services, unknown analyzers and arbitrary
metric substitution refuse. Keyword ranking/analyzer must be explicitly
implementation-defined with plain query syntax. Native HYBRID searches the full
indexed STRING inventory, so the authored keyword sources and admitted native
inventory must agree exactly. Native readiness and behavior require separate
managed evidence on the pinned target.

## Normative Surface

`compile_unity_search_json` is shared by the runtime, Python and WASM. Its closed
request retains original source document identity/hash/module/pointer, the
original raw-read selection and bound parameter declarations, projection, vector
parameter, topK, partial opt-in, managed target identity, catalog/schema/table UUID,
endpoint/index identity and native vector field/dimensions. Optional paired
keyword source/text parameter retain their own complete original source pins. HYBRID requires the exact explicit fusion object `{kind:"target-defined"}` and topK at most200; ANN refuses fusion.
`managed.nativeColumns` is required and retains the original ColumnSet Source
pointer plus `binding:{sourceId,documentJson,documentSha256}`. `sourceId` is the
selected input source alias, distinct from the binding document's internal ID.
The compiler checks model identity/hash/module/pointer, the ColumnSet target and
binding reference, `umf-binding-2` logical/target identity, the original Record's
native table and each Field's storage-column mapping. ColumnSet native types and
NOT NULL declarations must correspond exactly to the compiled current columns
and original row views, including other authored ColumnSets in that core source.
Missing source custody, wrong hashes/owners/types/nullability and ambiguous
native mapping refuse before an executable program is emitted.

The exact candidate fixtures and standalone original model/binding byte sources
are retained under `tests/native-search/` (`unity-model.json`, `unity-binding.json`
and the full numeric/Binary `unity-shaped-*` variants).

The compiler independently admits the complete original UMF model and selected
index declarations and unique native bindings. It refuses unsupported selected
meaning rather than dropping it. Current projection descriptors cover String,
Boolean, the selected indexed Vector, and explicitly shaped Integer, Decimal and
Binary; a required single String selected Key remains mandatory. Projecting the
Vector requires `managed.vectorBinding.nativeType` equal to `array<float>`. Outputs retain `carrier:"vector"`, `dimensions`, `nullable` and
`nativeType`; native double arrays refuse under CONTRACT-070; values must fit the exact float32
model carrier without rounding. Nullability derives from the unique original
Record view `row.null` and must match each bound scalar column. Relational
`row.missing` must be false; required Fields cannot admit null, and unspecified
availability refuses. An absent-allowed Field may retain a NOT NULL row view,
so core Field availability alone does not determine native nullability.

Additional `managed.projectionBindings` are exact `{field,native,nativeType}`
objects. Integer requires authored signed widths 8/16/32/64 matching
`tinyint`/`smallint`/`int`/`bigint`; unbounded, unsigned and other widths refuse.
Decimal requires authored precision 1..38 and scale 0..precision matching exact
`decimal(p,s)`; Binary requires `binary` and no unqualified facets. Output and
`currentTable.columns` descriptors retain `integerWidth` or `precision`/`scale`
as appropriate. Every numeric scalar binding requires a matching original shape.
No generated SQL casts or rounding repair a mismatched physical representation.
These are compiler candidate carriers; actual managed metadata and driver value
qualification remain separate required gates. Compiler source-custody controls do not replace the public SDK generator's full
bundle checks: an absent-allowed NOT NULL view is qualified only when every
authored binding ColumnSet carries the matching NOT NULL declaration. Raw table
config alone does not establish that premise.

Search input cannot aggregate, reorder, truncate or add unknown
query modifiers. Every predicate Field must occur in each selected index's
original `filterable` inventory.

The compiled program has version `weft.unity-search/0.1`, profile
`unity-catalog-current/0.1`, and a canonical structural digest owned by
`weft-databricks-search`. It retains the complete request and original metadata.
`query` contains an original REST method/path/body and explicit parameter-source
bindings. `hydration.statement` contains exact named native SQL and parameter
sources; candidate Key components use `{candidateKey:0}`. Hosts bind and execute
these statements unchanged and never invent SQL, predicates or fusion.

Each native candidate causes at most one bounded single-key read, conjoining the
complete original predicate with exact selected-Key equality and retaining the
original bound filter values. Native candidate order remains authoritative;
missing/now-nonmatching candidates increment stale and are not refilled. Duplicate
current Key rows, native denial, transport errors and malformed native responses
are errors, never stale hits. Reads return at most two rows so duplicate Key
violations can be detected before publishing a row.

## Host Obligations

- Require partial opt-in and report approximate/partial/asynchronous semantics.
- Validate the exact current native principal, table UUID, schema/carriers,
  endpoint/index type and readiness, selected Key, vector column/dimensions and
  complete STRING inventory before every database operation. Never query an
  unrelated existing business index to establish a fixture.
- Bind query vector/text as native values, validate vector finiteness/shape and
  original embedder identity, and use one timeout budget across all operations.
- Hydrate no more than topK candidates under current admission, withhold malformed
  or duplicate rows, preserve native ordinals, propagate native errors, and do not
  refill dropped candidates.
- Label native scores `implementation-defined`; do not invent an L2 distance
  formula or claim exact rank/score correspondence without separate evidence.

## Precedence and Compatibility

CONTRACT-074 governs authored semantics, current filtered hydration and result
reporting. Core source admission owns original model meaning; this backend owns
native statements; the host owns live native custody and execution. Existing
PostgreSQL exact search, raw Spark fixed snapshots and prior wheel APIs retain
their own profiles. SDK temporal qualification remains **HELD**.

## Error Semantics

Closed JSON, original-source, target/binding, budget, carrier, projection and
filterability mismatches return blocked diagnostics and no executable program.
Unknown managed behavior remains unqualified. A successful compile is not native
admission or a managed-service qualification claim.

## Validation Checklist

- [ ] Real owned DIRECT_ACCESS ANN/HYBRID fixtures and managed native admission.
- [ ] Stale deletion, changed tenant/category, no refill and native denial/errors.
- [ ] Wrong UUID/principal/index/vector/string inventory and source corruption refusal.
- [ ] Exact installed wheel and Chromium/WASM program parity.

## Prior Art

The [Databricks query-index API](https://docs.databricks.com/api/vector-search/v1/query-vector-index)
defines ANN/HYBRID requests, direct-access vector binding, result manifests and
ordered result arrays. Its `query_columns` override is beta; this profile retains
the full native STRING inventory instead. [Databricks vector-search documentation](https://docs.databricks.com/aws/en/vector-search/vector-search)
describes native HNSW/L2; normalization-dependent cosine equivalence is not a
generic cosine implementation claim.
