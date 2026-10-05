# B-002A application frontend evidence — 2026-10-05

**Outcome:** the versioned application-read frontend slice passes its native,
structural-schema, independent result and real-browser gates. This completes
B-002A's frontend scope. B-003 follows; public Python/compile-envelope and native
Truss/PostgreSQL or Ashlar/Databricks qualification remain later gates.

## Implemented contract

[CONTRACT-004](../../02-design/contracts/CONTRACT-004-application-reads.md) now has
an executable draft 0.2 grammar, typed IR, request schema and value-carrier schema.
Version 0.1 fixtures and behavior remain unchanged. A distinct test frontend
transport dispatches explicit `weft-sql/0.2.0`; it is not the public compile API.

Whole entities expand authored member order. Descriptors retain identities,
availability, ordered container item types and cyclic record-type graphs.
Selected unknown meaning refuses; owning-document source remains retained.
Absence and native null are separate: an optional scalar's ideal value type is
not rewritten into SQL nullable. Null envelope structural tests establish its
shape, not field permission or native decoding qualification.

Named entity/related-page profiles require a complete authored key, ascending
order, positive literal LIMIT through 1,000 and a complete lexicographic cursor.
The selected key identity is retained. Count-summary preserves join bags,
requires grouping for plain fields, and orders/limits complete grouping tuples.
A global empty count produces zero; grouped empty input produces no rows.
General 0.2 queries make no deterministic-page claim merely because they parse.

Relationship reads use authored module relationship IDs and explicit target
keys. Direct and inverse traversal preserve both original multiplicities and
lifecycle; source and target key descriptors remain explicit. Polymorphic,
undirected, association-record, unknown lifecycle, ambiguous and unkeyed selected
variants refuse in this initial subset. Existential predicates do not multiply
source bags; bounded keys retain duplicate tuples with exact truncation.

Typed parameters are exact strings with explicit families. Each use validates
its resolved field/key domain, so repeated uses intersect domains. Missing,
surplus, folded duplicate, mismatched and invalid lexical/domain bindings refuse.
No source value becomes a SQL fragment. Excluded trailing constructs are named
in application diagnostics. Type graphs, source parameters, projections, joins,
request bytes and SQL tokens have bounded paths.

## Executed gate

`sh scripts/run-b002a.sh` passed on macOS arm64 with Rust 1.90.0, CPython 3.12.14
(test oracle only), Node 24.19.0, Bun 1.4.2, Playwright 1.62.1 and real Chromium
153.0.8010.12. The retained [run log](b002a/run.log) records the complete gate
output; [source hashes](b002a/source-sha256.json) identify the inputs.

- **23 Rust integration tests** pass: 10 model, five application resolution,
  four application syntax and four original frontend tests. Corpus decisions
  cover all 636 original and 620 application cases. All 303 originally valid
  queries are also checked under explicit 0.2 general semantics.
- Application cases include **575** independently authored numeric boundaries
  reused as named parameter values. There are **304** resolved application plans
  and 316 expected refusals; every retained module bundle matches the input.
- **303 original and 304 application plans** pass their separate JSON schemas.
  All 620 application requests validate structurally when placed in the 0.2
  envelope with an illustrative fixture target. Presence/related carrier positive
  and negative schema cases distinguish malformed envelopes and preserve duplicates.
- The original independent bag oracle passes **13** result bags. The new
  independent Python/Decimal oracle passes **18** authored read-result scenarios:
  complete entities, optional absence/empty list/structure, global/grouped empty
  counts, join bags, exact decimal SUM, scalar/composite cursors, Unicode scalar
  and trailing-space key order, keys above binary64 integer precision, existential
  filtering, direct/inverse related keys and duplicate-edge truncation.
- Real Chromium produces byte-identical native reports for **636 + 620 = 1,256**
  requests. Separate [original](b002a/original-browser-summary.json) and
  [application](b002a/browser-summary.json) summaries record the same current WASM
  hash. Network APIs are disabled during calls; Node globals are absent; only
  wasm-bindgen reference-table/throw imports exist. No entropy, filesystem,
  database or network import is accepted.
- Specification checks pass: 43 governed artifacts, eight JSON schemas, 30
  planned public criteria and the unchanged 636-case original corpus.

Browser binary/linear-memory observations are not production size, peak-memory
or performance claims. The licensed ahash dependency configuration remains the
B-002 pure-WASM configuration; upstream warnings remain visible.

## Qualification limits and traceability

US-007-AC1 through AC6 have frontend component evidence here. Their full story
allocations remain planned: native carriers/explicit-null permission, ordering,
key stability, edge multiplicity and publication authorization need binding and
actual PostgreSQL/Databricks evidence. The oracle uses illustrative data and an
explicit duplicate-edge fixture, not a production mapping or enforcement claim.
The result carrier schema cannot authorize native-null states by itself.

B-003 must include these capabilities and typed plans in the backend interface.
B-004 must add the public 0.2 response/column decoding contract and real native
Python parity. B-005/B-006 require approved versioned Truss storage and Ashlar gold
bindings plus native execution access. Truss owns catalog/storage mechanics and
write protocols; Weft owns parsing, logical query semantics and backend lowering.
No Truss-owned design artifact is authored by this slice.
