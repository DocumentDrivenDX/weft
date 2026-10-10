---
ddx:
  id: TD-004
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: ADR-001
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
    - id: CONTRACT-004
      kind: informed_by
---

# TD-004: ashlar-databricks

**User Story:** [[US-004-ashlar-databricks]]. **Feature:** FEAT-002.
**Parent Solution:** [[SD-002]].

## Technical Approach

Inherit SD-002's separation of source meaning and physical lowering. Realize
US-004-AC1–AC4 through the shared pure Rust core and its registered backend/host
boundaries. Contracts 001–003 own exact interfaces and failure semantics.

## Component Changes

Target `crates/weft-core/`, backend integration packages and thin
`crates/weft-python/` / `crates/weft-wasm/` wrappers according to story scope.
Use one resolved plan, explicit capability evidence and exact value carriers.
Before implementation, pin dependency versions and finalize the IR/trait schema
listed in CONTRACT-001/002. Do not expand this story to invent database layouts.

## Interfaces and Security

Apply CONTRACT-003 input/pin/limit guards, CONTRACT-001 meaning and CONTRACT-002
plugin registration. Core has no I/O; plugins are trusted host code. No credentials,
policy decisions or untrusted SQL fragments enter the model-driven frontend.

## Testing

STP-004 names planned failing tests and assertions for every AC. Native profiles,
Python/browser behavior and independent expected values must receive evidence
at their actual layer; mocks/generated SQL never replace native semantic checks.

## Migration, Rollback and Implementation Sequence

No database migrations. Add fixtures/refusal tests first, implement the smallest
story components, execute the mapped evidence, then review conformance. A rejected
new profile/version rolls back by unregistering it; retained models are untouched.
Keep source-language/IR/backend versions separate and refuse stale caches.

## Candidate application-read implementation

CONTRACT-004 governs optional value envelopes, authored relationship resolution,
complete key order and bounded related results. The candidate adapter uses
Ashlar's pinned canonical edges or serving edges, with explicit original mapping
admission in `crates/weft-databricks/src/binding.rs` and traversal lowering in
`src/candidate/relationships.rs`. It derives logical keys from mapped UMF Fields
and retains physical IDs only for endpoint access. No database layout is added
to the compiler.

For US-004-AC1–AC3, whole-input endpoint/key/edge/multiplicity checks precede
user predicates; existential reads preserve source bags and ranked bound+1
lookahead preserves parallel edge tuples with an exact truncation marker. The
finite native ordinal domain is guarded explicitly. US-004-AC4 additionally
requires actual host verification of complete caller authority, immutable
publication, lifecycle and projection custody before execution and buffered
publication. The current native synthetic-admin corpus does not fulfill that
entire host gate. See the B-006 execution record for exact component evidence;
the [B-006 acceptance audit](../../04-build/evidence/B-006-acceptance.md) records
completed candidate recursive/presence lowering and buffered host fixture
enforcement, with production policy and release qualification kept separate.

## Optional-count feature boundary and configuration

The optional String COUNT/HAVING slice MUST retain the Core/adapter/runtime module ownership in Architecture. Core owns original optional availability, count/HAVING AST/IR identities and pre-binding qualification; the explicit CountHaving adapter owns finite target capacity and emitted native SQL/obligations; runtime owns registration; wrappers delegate. No compiler component owns native execution or public source/ACK enforcement. CountHaving may delegate reviewed lowering without exporting mutable internal target state to callers.

Configuration authority is the versioned request target, authenticated binding digest, original model pin vector and explicit build feature. These identities MUST remain separate. Environment variables, inferred SQL content and observed data MUST NOT introduce logical widths, native-null permission or backend selection. Invalid/missing/wrong profiles refuse through contracted diagnostics before callbacks; absence of a per-Field native-null encoding refuses. Old-profile behavior and artifact bytes remain governed by their original profiles. Runtime/toolchain paths are build-harness configuration outside library meaning; complete pinned build receipts are required for a realization claim.

## Formal Specification

The selected assurance level for this compiler-only slice is **precise-only with semantic review**, not model checking or a machine proof. CONTRACT-001–003 and the exact-arithmetic v0.3 appendix own the safety properties. Scope is one bounded compile request; external database state, scheduling, ACK/session continuity and host cleanup are excluded and require separate actual host/native evidence.

| Property | State/transition and guard | Implementation correspondence | Required verification |
| --- | --- | --- | --- |
| MEDIA-P1, CONTRACT-001/002 | Received -> parsed/resolved -> capability qualified -> binding validated -> lowered -> emitted. Unsupported optional-count/HAVING capability cannot enter binding callbacks. | Core `backend.rs` gate and owning-version resolver; explicit `count_having.rs` manifest | Zero-callback missing/unsupported/target/language controls, original-profile refusals |
| MEDIA-P2, CONTRACT-001/003 | Resolution retains optional argument identity/availability, own HAVING span and exact bounded literal; only the projected count's scan/Field may enter HAVING. | Core arithmetic query/resolve/IR; 1,024-byte expression token bound | Exact span/source-token controls; 1,024 acceptance/1,025 refusal; mismatched argument/parameter refusal; official schemas |
| MEDIA-P3, CONTRACT-002/003 | Lowering emits complete pre-HAVING source/capacity obligations; valid null arguments are absent only from count dedup capacity input, not from logical grouped source. | `candidate/arithmetic.rs` optional count lowering and unchanged source integrity | All-null/empty semantics, capacity SQL staging and exact source descriptors; native qualification separately required |
| MEDIA-P4, CONTRACT-003 | Emitted result is compiler evidence, never execution authorization or a native result. Unfulfilled obligations cannot be described as discharged. | Backend emission plus host boundary | Source/wheel/browser parity and retained obligations; separate guarded host/whole-interval tests |

Assumptions: trusted registered backend code, immutable exact prepared request/model/binding bytes, pinned schemas and bounded query resources. No fairness/liveness assumption or service availability guarantee is introduced: a compile either returns its contracted result/refusal or its runtime failure is captured separately. Input/component/bounds changes MUST rerun affected artifact, schema, callback and embedding checks. Actual engine proof cannot be inferred from this precise specification.

## Diagnostics applicability

For this pure compiler library, observability is the bounded structured Diagnostic/compile-result surface of CONTRACT-003: stable code, governed phase, severity, recoverability and original source span. The library MUST return it through the same public envelope across native/Python/browser paths, without independent logging or raw-value export. Thin CLI stdout remains protocol only. No live service, cross-process trace propagation, OTLP receiver, metrics backend or production telemetry claim is made by this slice. The host owns run identity, safe lifecycle/error capture, finite evidence storage and external execution diagnostics; its readiness and capture limits require separate review. Compiler regressions MUST retain schema-valid refusal examples and exact input/output custody in explicitly selected development evidence, including malformed configuration and resource refusal; a quiet console is not a pass.
