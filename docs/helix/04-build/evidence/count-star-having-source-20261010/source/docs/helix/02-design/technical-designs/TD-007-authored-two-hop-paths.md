---
ddx:
  id: TD-007
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: SD-001
      kind: informed_by
    - id: CONTRACT-005
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
    - id: CONTRACT-004
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# TD-007: Authored two-hop paths and exact grouped counts

## Scope and Technical Approach

Inherit SD-001's pure typed frontend and registered backend separation. Extend
US-007-AC2/AC3/AC4 through the draft CONTRACT-005 proposal, supporting the original
commerce order-line/product/supplier path without model edits, truncated-list
counting or inferred property relationships. PRD FR-1–3, FR-7–9 and FR-11–12 govern
versioning, bag meaning, capabilities, artifacts and conformance. US-007 is not
retroactively claimed to specify two-hop syntax: this extension requires interface
review before implementation. All proposal spellings and versions remain draft.

Use the same enumerated two-edge bag for bounded collections and relational
expansion. Keep distinct-neighbor constraint checks outside occurrence enumeration.
The selected candidate backend is local Spark/Delta under Ashlar's existing held
publication reader. Truss and Fabric qualification are independent; unavailable
access must not block the frontend/local lane or become simulated acceptance.

## Component Changes and Boundaries

Apply Architecture's Module Boundaries without new workspace edges.

| Owner/module | Owned change and public impact |
|---|---|
| `weft-core` new `path_syntax`, `path_ir`, `path_resolve` modules | Parse the selected version, preserve original spans, resolve exact authored identities and continuity, produce CONTRACT-005 typed path plans; no target/storage imports |
| `weft-core` `backend.rs`, `backend_emission.rs`, protocol routing | Assess capabilities before mapping callbacks; validate full path/count descriptors and new closed transport; preserve old emission/version checks |
| `weft-databricks` dedicated `path_lowering` module | Two-hop occurrence joins/ranking, exact collection carrier and aggregate lowering; reuse independently guarded authored relationship access without mutating one-hop output |
| `weft-runtime` | Explicit feature/profile registration and exact version selection; no SQL-based backend switching |
| `weft-python`, `weft-wasm` | Thin unchanged transport through the same core; new artifact schemas admitted explicitly |
| Ashlar host owner | Typed descriptor decoding, native integrity/count checks, original publication/authority/ACK custody and buffered result release; no compiler SQL repair |

Proposed modules are implementation locations, not claims that source exists.
Logical path types belong to core; native edge token encoding and target plan
belong to Databricks. Construction remains the runtime composition root, with
required caller-selected profile and existing original model/binding inputs.
`python3 scripts/checks/check-module-boundaries.py` and
`python3 tests/module-boundaries/check.py` enforce the Cargo map/private-access
boundary. Within-crate responsibility and semantic correspondence require review;
these commands do not prove them.

## Shared Interfaces, Configuration and Security

CONTRACT-005 owns exact syntax, typed IR/result, count semantics and new version
pairing; CONTRACT-002 owns registration and capability assessment; CONTRACT-003
owns pins, limits, diagnostics and result-release obligations. TD-007 defines no
alternative shared payload or adapter signature.

No persistent graph-table migration or synthetic model/Edge Record is needed.
Each hop consumes the original edge/object/key mappings. Authorize both endpoints
and edge populations under the original host context; inverse traversal grants
no broader rights. Untrusted names/models cannot introduce SQL fragments or code.

The compiler receives injected immutable request/profile settings and performs
no environment lookup. The host composition root constructs one typed runner
configuration using its existing owning configuration contract; add no hidden
operator defaults, credential reads or second owner for an existing key. Test
missing/invalid injection, isolation and redaction before native integration.

## Formal Specification and Correspondence

Assurance target is a reviewed precise specification plus implementation tests;
no machine proof is claimed. For a fixed accepted publication, define a starting
input bag S, independently guarded directed edge bags E1/E2 and complete endpoint
identity equality. Initialize no released result. Enumeration yields one tuple
(s,e1,e2,m,t) for each s in S with e1 starting at s and e1's target equal to e2's
source. Edge occurrences are unique; equal endpoint values do not merge them.

Transitions are guarded: admit original model/binding/profile; open one held
publication; validate consumed sources; enumerate or refuse; rank/count; decode;
close checks and readers; clean engine; release only after all preceding success.
Any failure enters withheld-result state. Retrying opens a new explicitly admitted
context and cannot continue a partial earlier result.

| Property | Authority and enforcing correspondence |
|---|---|
| PATH-S1: complete identity continuity and original model pins | FR-2, CONTRACT-003/005; resolver and native endpoint/source/type guards |
| PATH-S2: every legal edge pair appears once per starting occurrence; parallel edges survive | FR-3, CONTRACT-004/005; path lowering and independent finite bag oracle |
| PATH-S3: degree constraints count distinct associated instances, never path occurrences | CONTRACT-004/005; authored relationship degree checks and parallel-edge negative control |
| PATH-S4: bounded items are exact ordered prefix and marker iff total exceeds bound | US-007-AC2, CONTRACT-005; ranking/lookahead and bound boundary tests |
| PATH-S5: grouped path and distinct-terminal counts match the complete current bag exactly or refuse | US-007-AC3, CONTRACT-005; expansion/aggregate lowering and exact capacity checks |
| PATH-S6: no result crosses publication, authorization, decoder or cleanup failure | CONTRACT-002/003/005; Ashlar held reader and release finalizer |

Safety applies to every released result. Liveness is separate: for finite admitted
inputs and available terminating native/cleanup operations, execution eventually
releases a result or a bounded refusal. No guarantee is made for unavailable
engines, indefinite external failure, unrestricted path depth or production lease
fairness. The depth is exactly two; pure finite oracle tests do not prove native
termination or unlimited capacity.

If executable formal exploration is adopted, retain model/config/source hashes,
completed bounds, reachable successful/recovery witnesses and broken continuity,
bag-deduplication and early-release controls. A cutoff remains unknown. Actual
native execution and model/code correspondence review remain mandatory regardless.
Changes to path enumeration, degree/order/count, carrier or publication lifecycle
require rechecking the corresponding properties.

## Testing and Acceptance

Implement language-neutral fixtures with literal independently authored expected
identities, both edge IDs, key tuples, order, bags and counts. Compiler-generated
SQL or generated expectations are not the semantic oracle.

- Original commerce L1/P1/S1 passes collection, expansion/grouped path count and
  distinct supplier count on actual pinned Delta tables.
- Two first-hop and three second-hop parallel edges give six paths and one
  terminal; degree max-one passes. A second distinct neighbor violates max-one.
- Two intermediates reaching one terminal preserve two paths and one distinct
  terminal. Separate source/model identities with equal local IDs never join.
- Empty paths, exact bound, bound-plus-one and repeated equal key tuples prove
  ordered prefix/truncation. Empty global/grouped counts retain their own rules.
- Forward/inverse, self-join/repeated starting occurrences, ordinary join duplicate
  rows and filters preserve stage order and occurrence contexts.
- Ordinary LEFT joins distinguish an unmatched starting scan (tagged absent collection,
  zero expanded rows) from a present root with no paths (empty collection). Verify
  the actual scan-presence witness; nullable source properties cannot fake absence.
  Missing or forged presence evidence refuses before result release.
- Reusing the same edge in a forward/inverse return, and genuine self-loop edges,
  preserves each legal ordered pair rather than imposing an unstated no-repeat rule.
- Ambiguous/missing names, intermediate mismatch, unknown selected semantics,
  unsupported versions/capabilities, stale bindings, duplicate IDs, orphan/type/
  source mismatch, overflow, malformed carrier and unknown obligations refuse.
- Rust/Python/browser actual artifacts match under new schemas; old qualified
  artifacts and refusals remain byte/meaning compatible.
- Actual Ashlar host tests cover ACK/authorization/publication drift, decoder
  rejection, reader close and engine cleanup failure with no report/result release.

## Diagnostic Wiring

Reuse CONTRACT-003 bounded diagnostics and original source spans. No new event
schema or telemetry exporter is introduced by this pure compiler slice. Host
capture uses its governing diagnostic contract, keeps protocol stdout clean and
never logs credentials, raw keys or source values. Run/attempt attribution must
work without an active trace span. If changed host instrumentation/export is
needed, first specify its OpenTelemetry mapping/version and verify a real receiver,
privacy, bounded capture, loss and shutdown; JSON lines are not receiver evidence.

## Sequence, Compatibility and Risks

1. Review CONTRACT-005 spelling/version/token decisions and finalize closed schemas;
   add failing semantic/version/refusal fixtures and their independent oracle.
2. Implement pure typed core resolution and descriptor validation; verify version
   fences, capability-before-callback behavior and unchanged old artifacts.
3. Implement dedicated Databricks lowering and separately admit its profile and
   immutable compiler realization; verify exact module boundaries and wrapper parity.
4. Execute tiny actual collection and grouped-count witnesses through Ashlar's
   publication reader, including the parallel-edge and genuine failure controls;
   independently review complete original artifact/native/ACK custody.

Rollback unregisters only the new profile/realization; original models, tables,
previous compiler packages and older profiles remain intact. Reusing a correlated
aggregation by logical relationship alone risks merging different starting scans;
access caches must include occurrence context. Sorting encoded numbers as strings
risks changing key order; use exact typed ordering before carrier serialization.
Expansion increases bag size, so native count/resource bounds require explicit
proof or refusal, never hidden truncation. The initial commerce profile encodes original native BIGINT edge IDs as canonical
signed-decimal tokens ordered by their decoded integer value; admission must
verify real native order, reversible encoding and original identity guards.
Interface review must settle the restricted path alias and closed schemas before
implementation readiness.

## Required-root related-key profile realization

The separate profile in CONTRACT-005 composes unchanged Plan04/Backend03 types
and exact inherited RelatedKeys representation with the existing two-hop/scalar
capabilities. `weft-databricks::paths_keys` owns its named backend manifest and
admission; `weft-runtime::paths_keys` owns feature-selected registration and the
`weft-paths-keys` entrypoint. The existing paths route never auto-selects it.
Keep shared binding/relationship access and collection lowering private to
Databricks, with explicit selected-profile policy rather than a SQL-triggered
fallback. Core emission validates original output/relationship/key lineage and
complete obligation inventories before emitting an admitted response. Native
source-schema declarations are immutable metadata; host observations prove their
actual correspondence.

Extract or reuse the exact two-hop DECIMAL38 prefix operation for one-hop
collections. Do not call the historical Traversals::related ROW_NUMBER algorithm
for this profile. Access caches include original relationship identity and root
occurrence; full authorized bag enumeration preserves parallel edges while degree
checks retain distinct-neighbor inputs. The new ordinalCapacity inventory covers
both selected one-hop and two-hop bounded outputs before any lookahead. Keep
signed64 aggregate capacity, source integrity and encoding proof separate.

Configuration has one owner per field: runtime/build selects the named feature;
trusted application composition selects the registered backend/version/profile,
immutable index and platform qualification; operators inject deployment handles,
credentials and finite capture limits through the host's typed startup config.
Compiler/lowerer reads no ambient environment. A library request cannot provide
its own trust authority; no mutable global registration or new dependency edge
is needed.

Extend the precise specification with a held admitted edge bag E for each root
occurrence s. One-hop enumeration contains (s,e,t) once for each legal e; its
ordered prefix P is take(bound, E ordered by complete target key then edge ID),
and truncated iff |E|>bound. Initial state has no released result. Admission,
source/schema checks, full wide-prefix capacity, encoding, decode and closure are
required transitions; any failing transition withholds release. PATH-K1 preserves
original identities and bag multiplicity; PATH-K2 establishes exact order/prefix/
marker and full DECIMAL38 overflow refusal; PATH-K3 preserves PATH-S6's publication
and cleanup fence. Assurance remains precise semantic review plus implementation
and actual native correspondence tests, without machine proof. Liveness assumes
finite admitted inputs and terminating available native/cleanup operations;
no production fairness, crash-durability or unbounded-fanout claim follows.

Reuse CONTRACT-003 safe bounded diagnostics. Attribute profile, stage and outcome
without logging keys, SQL values or credentials; stdout remains protocol-only.
Known capture loss/cleanup failure must remain visible and primary cancellation
must survive cleanup. This change adds no OpenTelemetry exporter or receiver
claim; host instrumentation changes require its own exact mapping/receiver proof.

Verify contract/descriptor and forbidden combinations first, then backend manifest
and lowering correspondence, then fresh immutable compiler transport/package
admission, and finally the actual original ten-case UMF0.8 commerce workflow.
The required RelatedKeys case must pass under the new realization; another
profile's collection/property-join result cannot substitute. Recheck PATH-S1–S6
where shared prefix/access changes affect two-hop behavior. Native claims retain
exact compiler/source/backend/runtime/publication/ACK identities.


## Grouped projected row-count HAVING implementation slice

Apply CONTRACT-005's grouped row-count extension through the existing pure core
and registered PathsKeys boundary. No Ashlar query repair, source conversion or
new backend dependency belongs in this slice.

| Owner | Narrow change |
| --- | --- |
| Core `arithmetic_query.rs` | Represent HAVING argument as distinct Field or row-count token/span; parse the selected new dialect without widening old versions |
| Core `arithmetic_application_resolve.rs` | Resolve nonempty exact groups and explicitly projected row count; retain distinct resolution and original literal/span meaning |
| Core `backend03_emission.rs`, version routing and v0.4.1 schemas | Validate new capability/expression correspondence; isolate versioned schema additions from v0.3 and older emitted artifacts |
| Databricks `paths_keys.rs` | Own new candidate version/capability selection; other registered profiles refuse the addition |
| Databricks `candidate/path_lowering.rs` | Lower typed row-count HAVING using a bound BIGINT threshold and existing complete pre-HAVING count guards; distinct lowering remains unchanged |
| Runtime/Python/WASM distribution owners | Build the same Rust core, refresh explicit registration/manifest/index hashes and verify immutable public admission |

Keep AST discrimination inside the core; the backend consumes typed views rather
than importing parser internals. Construction remains trusted runtime registration;
operators/hosts retain deployment and publication ownership. No ambient settings,
mutable plugin registration, credential access, new telemetry field or exporter
is introduced. Existing bounded diagnostic stage/code/span projections suffice;
raw query values are not new log attributes.

The precise semantic review uses state `unreleased` initially and transitions
parse -> resolve -> assess -> lower -> source/capacity guards -> query -> decode
-> closure -> release. HC-S1: every released group has exact original key and
count C(g), and C(g)>N. HC-S2: every selected source row and pre-HAVING count is
validated before predicate filtering; an overflowing excluded group still
refuses. HC-S3: unsupported versions/profiles and every failed guard withhold
release. Map HC-S1 to resolver/lowering and duplicate/empty oracle cases, HC-S2 to
the actual `ashlar.arithmetic.exact` / `aggregate-candidates` inventory for
plain grouped COUNT(*) and complete-group overflow hidden by ORDER BY/LIMIT plus source-invalid rows
excluded by HAVING negative controls.
`ashlar.path.countCapacity` remains expansion-specific and is checked only when
the plan actually selects expansion. Map HC-S3 to
registered capability/version checks and host closure obligations. Assumptions
are finite admitted bags, established exact String equality, one held complete
publication and terminating native/cleanup operations. This is a precise semantic
specification and bounded implementation correspondence review, not a machine
proof or arbitrary-scale liveness claim. Recheck if grouping, count representation,
join bag semantics, capability routing or release guards change.

Implement only after contract/schema/version review. Add original replay SQL and
boundary/refusal controls before parser/resolver changes; then validate lowering
and complete guard correspondence. Verify Rust, native Python and browser WASM
public compiler parity with fresh packages and retain byte-identical old distinct/artifact
snapshots for requests selecting existing versions and profiles. New 0.4.1
artifacts may carry required version/realization marker changes while preserving
prior distinct semantics and expression shape. Native tests are a later independent qualification: compiler acceptance
alone cannot establish host custody or engine results. Rollback selects the
previous immutable distribution/version and restores its explicit refusal;
no data migration or publication/ACK authority change is required.
