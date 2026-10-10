---
ddx:
  id: TP-001
  type: test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# Compiler conformance test plan

## Testing Strategy

Goal: establish that Weft SQL + supplied UMF modules produces target SQL with the defined logical meaning, or a precise refusal. SQL snapshots alone are insufficient. Every support claim names compiler, model, backend, storage profile, engine version, settings and evidence revision. B-001 through B-006 now have scoped compiler, native and embedding evidence;
the B-007 acceptance matrix records passing evidence for all 30 P0 compiler criteria at the exact profiles and observed hosts in the support inventory. Distribution remains separately gated.

### Test Levels

| Level | Coverage target | Priority |
|---|---|---|
| Contract | Every grammar production, type rule, diagnostic, request/response schema and capability decision | P0 |
| Unit/property | Exact numbers, identity resolution, bag algebra, bounds, stable serialization and hostile input | P0 |
| Backend integration | Execute emitted SQL over native PostgreSQL and Databricks fixtures; compare typed row multisets | P0 |
| Cross-runtime | Rust, installed Python extension and real browser WASM compile the same requests | P0 |
| Plugin integration | Third synthetic backend registers without modifying the frontend; refusal and trust boundaries | P0 |

### Frameworks

Rust test runner plus a pinned property-testing library; Python pytest; browser automation through a pinned Playwright version; actual PostgreSQL and Databricks SQL clients owned by the test harness. Select exact versions in SPIKE-001 and record them before implementation. Bun runs documentation integrity checks only.

## Test Data

The committed corpus contains 636 independently specified scenarios, including all signed and unsigned widths 1–64 at both valid boundaries and adjacent invalid values, decimal boundaries, syntax and semantic refusals, exact-string distinctions, module ambiguity and pin failures. It is input and expected behavior, not test execution evidence. Synthetic binding profiles are explicitly illustrative; production adapters require separately accepted storage mappings.

Use independent arbitrary-precision arithmetic and a small bag-relational oracle that does not reuse compiler lowering. Native results use exact text transport for numbers. Compare unordered multisets, preserving duplicates; never sort numerically through floating point. Null is a tagged value. Include empty aggregates, high fanout, duplicate keys, negative cancellation, signed bounds, precision overflow, combining marks, emoji, trailing spaces and hostile identifiers/literals. Retain unknown extension content byte-for-byte while refusing unknown selected semantics.

Factories generate small models and rows with fixed recorded seeds. Shrink failing cases and commit minimized regressions. Mocks may test plugin failures, but do not establish native target support.

## Coverage Requirements

| Gate | Minimum | Enforcement |
|---|---|---|
| Story acceptance | All 30 P0 criteria have passing named tests | Traceability audit; no planned-only criterion qualifies |
| Initial corpus | All 636 cases evaluated, deliberate structural negatives included | No unexplained skip or changed expected output |
| Expanded release corpus | At least 1,000 distinct assertions across frontend/backend/runtime matrices | Counts are a floor, not evidence of semantic completeness |
| Property checks | At least 10,000 deterministic generated cases per release plus persisted regressions | Record seed, generator version and failures |
| Native adapters | Every claimed supported operation/type/domain executes on each claimed engine profile | Candidate or blocked when access/evidence is unavailable |
| Critical semantic branches | Every supported and refused branch has an assertion | Branch report and review; a line percentage cannot substitute |

P0 paths: model pins and identity; parse/resolve/type; exact numeric overflow; multiplicity and empty aggregation; safe SQL/parameter emission; missing/null/retained storage boundaries; runtime parity; policy/publication obligations. P1 adds larger query compositions and performance measurements within documented limits. Timing thresholds require an agreed environment rather than invented SLAs.

## Acceptance Criteria Layer Allocation

| Criterion class | Story plan | Primary layer |
|---|---|---|
| Language/model semantics | STP-001 | Contract + independent oracle |
| Registration/capabilities | STP-002 | Plugin integration |
| Truss mappings | STP-003 | Native PostgreSQL integration |
| Ashlar mappings | STP-004 | Native Databricks integration |
| Host embeddings | STP-005 | Cross-runtime end-to-end |
| Version/evidence/unknowns/bounds | STP-006 | Contract/property |

Each STP owns its four criterion rows; `story-test-allocation.json` records the 30 stable IDs and planned test names. Test implementation must carry those IDs. Allocation does not imply coverage.

## Implementation Order

1. Schema validation and independent oracle before compiler behavior.
2. Frontend and synthetic plugin against positive and refusal fixtures.
3. Native embeddings and third-backend seam.
4. Owner-approved storage profiles, real adapters and native execution.
5. Expanded property, mutation, fuzz and release qualification.

## Infrastructure

CI design: pinned Rust/Python/Bun/browser versions; isolated disposable PostgreSQL databases; isolated Databricks test catalog and compute with explicit cleanup and cost limits approved by its owner. No production data or credentials in fixtures. Native tests must report unavailable distinctly from pass; release qualification requires access to both initial targets. This bootstrap does not provision CI or compute.

## Risks

Engine numeric ranges, aggregate widening, collation and session settings can diverge. Test boundaries on actual engines, declare domains, and refuse unsupported semantics. Production Ashlar layout and Truss binding/version profiles remain owner decisions. The fixture profiles must never be promoted to production evidence. Shared implementation in an oracle can conceal bugs; use separate arithmetic and native engines. Mutation tests should demonstrate detection of dropped type filters, duplicate elimination, rounding, and absent/null substitution. Fuzz parsers and JSON limits; record minimized inputs.

## Build Handoff

The executable command `bun run specs:check` validates documents, schemas, initial planning allocations and fixture integrity only. Rust, native and host commands now exist and are pinned in the build evidence. The [B-005 acceptance audit](../04-build/evidence/B-005-acceptance.md) maps the Truss compiler criteria to actual native/compiler evidence; it does not qualify the release matrix. Release blocks on unresolved P0 criteria, missing native evidence, semantic mismatches, unreviewed binding profiles or undocumented skips. A candidate artifact can be explicitly requested for experimentation; it is not release conformance.

## Application-read extension

STP-007 allocates the six PR #2 outcomes to frontend/oracle, host and native-backend layers. Preserve 0.1 corpus expectations and add separate 0.2 profile fixtures; no planned criterion is counted as covered by incorporation into the PRD.


## Concrete CLI distribution qualification

Run the actual produced release executable with its exact selected feature set,
not an instrumented test binary. Freeze the complete ordered tracked-source and
public-schema/backend inventories; record actual tools, commands, features, target
and known environment, retaining unknown build properties. Execute every declared
public-transport case with complete response and deterministic-repeat comparison.
Keep original receipts; allow only independently reviewed exact historical
expected-artifact migrations with old/new path, value and digest correspondence.

Add current malformed envelope/pin/binding/resource, candidate-opt-out and wrong
backend/profile controls. Vary valid fresh publication IDs, UUIDs and versions;
reject static inconsistent vectors at compile time and actual native drift through
host observations. Coherent retained older publications remain usable under host
retention rules; registration is never bound to fixture values.
Verify byte-cap, multibyte UTF-8, input/output I/O failure and framing on the actual
executable. Assert complete unique case IDs and executable identity around every
invocation, then close source/harness/corpus custody before reporting success.

Map installation tests to DIST-F1–F4 and the conditional DIST-L1 progress witness.
Exercise production/index admission separately from trusted consumer installation,
including adversarial changed binary/record/schema/corpus,
wrong target/features/backend, missing/partial receipts and unindexed manifests.
None may register itself or fall back to another compiler. Retain safe structured
stage/refusal diagnostics; check that credential and raw request payloads are
absent. A separately executed fresh native publication/read from documented
commands is required for the consumer's setup claim. Compiler transport evidence
cannot substitute for that native execution.

## Separate paths-keys profile verification

CONTRACT-005 and TD-007's PATH-K1–K3 require 100% coverage of selected supported
and refused branches. Contract tests use the exact 0.4/Backend03 pair, inherited
closed RelatedKeys carrier, original complete key/pin/output-position lineage and
closed collectionIntegrity/ordinalCapacity inventories. Test required String
composite keys, inverse names, repeated outputs, missing/stale/forged edge schemas,
unknown/duplicate obligations and LEFT-root/aggregate/expansion-mix refusals.
The owning collectionEncoding check must reject one NULL original edge ID and
duplicate original edge IDs in the complete consumed source, including an edge
outside the bounded prefix and an outer selection returning no rows. Native
schema observation remains separate from these full-source value controls.
The old paths manifest/route retains its one-hop refusal; the new manifest has
exactly its previous 47 capabilities plus boundedKeys, without silent registration.

Unit/lowering correspondence tests use independent tiny edge bags for empty,
exact-bound, bound-plus-one, equal-key parallel occurrences and distinct-neighbor
degree. One-hop and two-hop prefix SQL must use the full DECIMAL38 TRY_SUM ordinal,
with full-population NULL overflow checks before lookahead/outer LIMIT. Verify
emitted expressions and independent arithmetic boundary witnesses; do not allocate
10^38 rows or claim an actual enormous-population test. Guard omission/misordering,
dropped parallel edges, wrong key order and hidden overflow are negative controls.
PATH-K1–K3 remain precise-review properties; no analyzer pass is implied.

Native integration independently compares full raw-source bags, exact collection
cells, ordering/truncation and schemas under one held publication with ordinary
protected ACK. Run the actual original UMF0.8 ten-case commerce workflow, including
required one-hop, two-hop multiplicity/distinct/grouped counts, property joins and
settlement/refund arithmetic. Retain source-public-UMF, parameter slots, actual
schema observations, guard outcomes and closing custody; inject query/cancellation,
reader/ACK/resource drift and cleanup failures to withhold release. Keep separately
authored parallel/signed-edge/self-loop/inverse/LEFT witnesses distinct from that
original source. Other compiler/native receipts cannot qualify the new realization.

Distribution tests execute the new release binary's complete declared corpus,
selected/refused capability map, namespace fences and bounded transport controls;
index admission and installed fresh-process use remain separate. Tests verify
required typed configuration and permitted/forbidden dependency edges with the
existing boundary commands. Diagnostics must preserve primary failures, bounded
safe stage evidence and explicit capture loss without credential/value leakage;
local records alone claim no OpenTelemetry receiver integration or production
retention. No renewed scale experiment is required for these finite semantic
and representation-bound controls.
