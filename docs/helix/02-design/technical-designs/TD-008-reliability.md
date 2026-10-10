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

## R2 implemented module ownership boundary

The repository map [module-boundaries.json](../module-boundaries.json) assigns responsibility, retained types/public declarations, construction and integration ownership to all eleven workspace packages. Core owns semantic preparation, exact literals and blocked simulation; backend packages consume core ports, runtime constructs registrations, and Python/WASM hosts call runtime. The original B001 spike and probe packages remain separate test-owned islands. The direct-edge allowlist includes external, optional and dev dependencies; unexpected edges, packages, source files or public declarations refuse. Internal workspace edges must form a DAG, even when each edge is individually allowed.

`bun run modules:check` executes the same Cargo metadata check and negative controls locally, from `.githooks/pre-commit`, and in CI. Install the hook with `git config core.hooksPath .githooks`; a checkout without that local setting still has the documented command and CI gate. Cargo is selected through the host configuration seam (`WEFT_CARGO`, required explicit executable handle supplied by the typed runner configuration or CI toolchain setup). The checker consumes real manifests with locked/offline metadata. It does not infer imports from grep or claim a complete Rust type-level or intra-core architecture proof: public declaration inventory is a conservative lexical ownership check. Renames, added surface, dependencies or packages require an explicit reviewed map update.

Fifteen test cases exercise allowed edges, forbidden optional edges, unowned and missing packages, cycles with otherwise allowed edges, added/removed public types, new sources, actual file-scanner bypass forms, unsupported visibility forms, same-name source substitution through real Cargo metadata, directory symlink refusal, const-unsafe API identity and all JavaScript dependency groups. Explicit Rust #[path] references, including raw literals and comments, are lexically inventoried and must resolve inside the checkout. Canonical manifest/path ownership and direct dependency source/version/alias/target/features are checked. Browser wrapper and host-tooling entries own construction and integration with automated file inventories and explicit manual import/semantic review; generated browser dist files are excluded from source ownership. A documented map alone is insufficient evidence; the actual repository check must also pass.

## R3 host configuration and mutation implementation

CONTRACT-006 governs the central immutable Python Config. `scripts/reliability/config.py` is the one runner configuration provider; `cargo.py` validates the pinned Cargo/Rust toolchain and invokes it through the bounded process boundary. `tests/qualify-and-evolve/source-mutations.py` consumes that provider and never contains machine-specific operational defaults. The boundary checker requires an explicit Cargo handle; CI supplies it from its installed toolchain. Python standard-library dataclasses, argparse and JSON supply typed validation without adding a compiler dependency.

The mutation catalog is repository-owned in `scripts/reliability/mutations.py`. Copies are fresh and isolated, replacement cardinality is explicit, and a clean original-source baseline (exactly one executed/passed test, no failures/ignored tests) precedes each edit. Its outcome is retained. A detection requires exactly one executed/failed selected test plus exit101 and every expected panic/signature. All decisions use explicit conditions under normal and optimized Python. Subprocess output is drained in8192-byte chunks, with at most65536 bytes for line parsing and4096 bytes of signature overlap; only safe numeric counters and known signature identities survive. Unix process groups receive TERM/KILL on timeout with at most two seconds of drain/kill grace and one second of wait. This Unix host boundary is scoped to macOS/Linux; Windows runner execution remains unqualified.

R4 adds the governed lifecycle/retention/export/retrieval behavior. R3 does not claim that a failed runner has complete durable capture yet; it returns nonzero and never writes a passed summary after a mutation failure. Existing raw B007 mutation logs remain immutable historical evidence.

Astra R3 review exposed source-root recursive copies, effective compiler overrides, unbounded unsafe version probes and already-failing baseline false detection. Canonical overlap refusal, explicit RUSTC/disabled wrappers, bounded exact version probing and clean baseline gates repair those cases. Evidence records the repaired executions separately from earlier exploratory runs.

## R4 host diagnostics implementation

`diagnostics.py` owns the safe Python logger bridge, actual OTel SDK records,
version-pinned OTLP JSON encoding, bounded local capture and export queue, lifecycle
manifest and closed-run retention. `retrieve.py` exposes read-only snapshot pages
with authenticated owner-scoped cursors. `source-mutations.py` and `cargo.py`
construct diagnostics only after typed configuration/tool identity admission and
record actual operation outcomes; incomplete capture/export refuses qualification.
Compiler and browser libraries do not import these host modules or SDK packages.

The receiver pilot decodes actual HTTP requests with the pinned official OTLP
protobuf messages. It checks real SDK span/log context, an out-of-span event,
wire enum/int64/hex representation, unsampled attempt counts, duplicate sequences,
privacy at all projections and failed-attempt retrieval. Failure controls exercise
local capture denial/gaps, queue/record overflow, outage, bounded shutdown,
retention, malformed/symlink input, cursor integrity/expiry and replacement.
Recorded pilot timing is a scoped local observation, without a production SLO.

## R6 qualification ownership

`custody.py` inventories repository compiler, bindings, schemas, corpora, oracles,
tests, runners, vendor sources, build and lock/config/checker inputs plus governing
HELIX artifacts. Exact named generated evidence outputs and build products are
excluded; Rust embedded inputs must be present in the inventory. Changed, removed
or new relevant files invalidate the manifest. B-007 source hashes resolve only
against immutable checkpoint f81565a1addaa6d2c83561f62d3805d1167233ee.

`fresh_hosts.py` uses CONTRACT-006 configuration, explicit pinned host-tool
handles and safe lifecycle capture. It builds a public qualified CLI, native wheel
and WASM, installs that exact wheel, and runs the actual Chromium compiler. The
complete2181 ordinary corpus retains full-response byte parity to fresh CLI and
semantic parity to retained native artifacts. Seven resource and six blocked
security cases cover all three hosts. The input snapshot is verified unchanged
before a passing qualification is written. CI runs the same gate at the final
pushed SHA, alongside both Rust feature compositions, conflicts, retained replay
and controls. Successful replay is historical custody, never current native
authority, security enforcement or a release claim.

Qualification helpers execute with isolated nonoptimized Python (`-I`), and
legacy assertion-based helpers explicitly refuse optimization. Loaded native
extension bytes must match the exact fresh wheel member; all2181 receipt IDs,
counts and expected/actual response hashes must join to fresh CLI. Built CLI,
batch, wheel, WASM/glue and wrapper hashes are frozen and reconciled. Scope is
repository inputs plus pinned dependency/tool identities, not arbitrary dynamic
build-script IO. Unsupported Rust include forms and nested schema attributes
and unknown path-bearing attributes refuse instead of hiding a dependency. Historical manifest membership is anchored
to its original digest.

CONTRACT-006 bounded raw-stream scanning describes the direct diagnostic process
boundary. Fixed-corpus parity helpers retain complete authored protocol artifacts
through their existing captures; those captures have outer deadlines and are not
SDK diagnostic sinks or a general adversarial-output memory guarantee.
