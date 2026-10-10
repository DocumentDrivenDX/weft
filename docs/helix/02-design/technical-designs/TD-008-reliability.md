---
ddx:
  id: TD-008
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-008
      kind: informed_by
    - id: SD-004
      kind: informed_by
    - id: ADR-001
      kind: informed_by
    - id: ADR-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# TD-008: Reproducible compiler qualification

**Story:** [[US-008-reliability]]. **Parent solution:** [[SD-004-conformance-evolution]].

## Technical Approach

Realize US-008-AC1–AC7 in B-009 slices R0–R6. Apply installed HELIX 0.15.4
modularity, rust-cargo configuration, o11y-otel and formal-methods practices.
The pure Rust compiler receives explicit requests; only host tools perform IO,
read environment/configuration, capture evidence and export diagnostics.

## Component Changes and Boundaries

R1 bounds runtime stdin to CONTRACT-003 request bytes plus one sentinel before
UTF-8 parsing. Catalog keeps retained input and parsed meaning inseparable;
callers receive only shared input slices. Re-preparation is required after changes.
R2 adds a reviewed package/type/API/construction/integration ownership map and
checks actual Cargo dependency metadata, unknown packages/edges and cycles.
Rust visibility enforces private state. Semantic intra-crate coupling remains a
named review obligation; the package checker cannot prove semantic independence.
Local, pre-commit and CI invoke the same boundary check.

R3 centralizes runner configuration with typed defaults → optional file →
allowlisted environment → CLI precedence. Explicit operational paths/handles
have no committed defaults. Unknown/malformed keys refuse before side effects.
A pinned toolchain wrapper supplies executable identity; mutation experiments
use fresh isolated copies, exact replacement cardinality and explicit failures.
No assert may decide a Python harness qualification outcome. Unapplied mutation,
missing selected test, wrong panic/signature and optimized Python must fail closed.

R4 captures only allowlisted lifecycle fields before local, console and exporter
sinks. Raw subprocess streams are excluded by default, drained without retaining
unbounded bytes. Bounded records/retention/queues and operation/shutdown deadlines
surface loss, outcome and crashes through an atomic run manifest. Read-only paged
retrieval identifies expired cursors and concurrent incomplete runs. An optional
OTLP/HTTP JSON route uses official OTel data-model fields and real local receiver
proof; missing export or local capture remains observable. No synthetic trace or
span IDs are invented outside actual spans. Exact new configuration/events and
cursor surfaces must be governed by a runner Contract before implementation.

R6 builds fresh qualified CLI, native wheel and browser WASM, records exact loaded
binary identities and compares whole canonical responses against the retained
2,181-case ordinary corpus plus blocked security controls. Input manifests cover
source, schemas, bindings, corpus/oracles, test/harness/build tools and locks;
changed, missing or new relevant files invalidate qualification. Historical B-007
custody resolves the declared Git checkpoint without rewriting original hashes.
Both valid Rust feature compositions and intentional conflict refusals run.
Successful required hosted CI jobs at the final pushed SHA are a final gate.

## Interfaces and Security

CONTRACT-003 owns CLI/compiler boundaries; CONTRACT-005 retains closed security
activation. New runner interfaces belong in a separate reliability Contract.
Credentials/native authorization stay with separately invoked native hosts.
Diagnostic fields never contain query/model/binding/parameter text or raw streams.
A capture/export failure cannot silently convert an incomplete run into qualified
support. No new service/SLO or database provisioning is included.

## Testing

STP-008 allocates exact outcome rows. Each executable covering test must cite its
AC ID. Broken controls use the real checker/runner, including deletion, forbidden
edges/cycles, optimized Python, capture/export failures, queue overflow and timeout.
Full fresh artifact parity proves compiler transport on recorded platforms, not
fresh database execution, current stored-data validity or production security.

## Trade-offs, Migration and Rollback

Catalog encapsulation intentionally breaks mutable public-field callers; migrate
them to inputs() and explicit preparation. Runtime input growth is capped at the
contract boundary. Central configuration adds one host-only dependency seam,
keeping ambient state out of the compiler. Evidence outputs remain separate from
qualification inputs to avoid self-referential hashes. No database migrations.
Revert a failing chunk or decline qualification; never relabel historical evidence.
B-009 defines ordered paths/commands and review gates. Platform release matrix and
native production qualification remain open questions.

## Diagnostic Proof

STP-008/R4 requires actual receiver wire mapping, privacy across all sinks,
duplicate prevention, records without spans, bounded outage/shutdown and cursor
expiry/concurrency. Pilot fixtures have known lifecycle ground truth; measure
captured/lost event counts, retrieval calls/bytes and operation overhead against
project-owned limits fixed in the runner Contract before implementation.
