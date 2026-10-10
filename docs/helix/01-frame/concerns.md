---
ddx:
  id: weft.concerns
  type: concerns
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
---

# Weft concerns

## Active Concerns

| Concern | Source / selection | Areas | Practices |
| --- | --- | --- | --- |
| rust-portable-core | Project-local; language-runtime operator override of shipped TypeScript default; discussion-backed proposed direction, ADR-001 | all | Pure Rust core; exact JSON/text boundaries; native Python plus browser WASM; toolchain pins follow spike |
| compiler-with-plugins | Project-local; architecture-style assumption grounded in explicit pluggable-backend direction, ADR-002 | frontend, backend | Resolved typed IR independent of physical SQL; explicit host registration |
| UMF fidelity | Project-local; owner direction | model, frontend, backend | Preserve unknown content; selected unknowns block; native/core claims remain distinct |
| Exact host transport | Project-local; Python/browser requirement | bindings, backend | Integers/decimals as lexical strings; declared overflow/presence/comparison behavior |
| Bounded untrusted processing | Project-local | all | Input limits, no fetching/code loading, deterministic diagnostics and cancellation boundary |
| Evidence and conformance | Project-local | all | Independent expected results; real native execution; version/subset qualifications |
| modularity-and-encapsulation | baseline-policy; handwritten Rust/TypeScript/Python present; owner: Weft owner | all | TD-008 boundary adoption/checker R1/R2; language visibility; semantic ownership review; no baseline exclusions |
| rust-cargo | Owner-requested configuration adoption; owner: Weft owner | all | Centralized host configuration and pinned toolchain wrapper; explicit scoped overrides below |
| o11y-otel | Owner-requested diagnostics adoption; owner: Weft owner | all | Host runner only; OTel mapping, real receiver proof, privacy/bounds/loss/retrieval; exact versions governed before implementation |
| formal-methods | Owner-requested assurance adoption; owner: Weft owner | all | TD-009 executable analysis for finite composition; precise specification for activation/custody; no native/refinement proof |

## Project Overrides

Rust fills the language-runtime slot for the compiler, rather than HELIX's shipped
TypeScript/Bun default (ADR-001). Bun may run documentation and JavaScript wrapper
tooling; it is never required by Python consumers. No datastore/deploy/auth/UI slot
applies to a pure compiler library. These selections are proposed, not evidence
of a working Rust implementation; the embedding gate can revise tool choices.

### B-009 scoped practice overrides

Authority: owner-requested reliability remediation, B-009 and accepted ADR-001/002.
Owner: Weft owner. Review/removal trigger: source boundary changes, activation or
release of security, a new configuration/diagnostic surface, or platform expansion.

- **modularity-and-encapsulation:** Applies to all handwritten code. Existing
  source has no complete boundary map/checker; R1/R2 are bounded baseline adoption
  work. R2 must supply architecture map, allowed/forbidden edges, construction
  policy and real controls before dependent feature work. No exception or widened
  baseline hides an existing/new violation; semantic checks remain review-owned.
- **rust-cargo:** Adopt central validated configuration and pinned invocation for
  B-009 host tooling. Compiler libraries stay pure with explicit request inputs.
  Existing Rust 1.90.0/edition 2021 follows ADR-001; the older workspace's entire
  lint/error-library/profile/edition/dependency-policy migration is outside B-009.
  Do not claim the full shipped stack checklist is met. Host tools use typed Python
  dataclass configuration and stdlib structured logging rather than adding Rust
  figment/secrecy for Python-owned runner IO. Alternative verification: precedence,
  unknown/malformed/missing keys, no ops defaults, privacy and pinned version tests.
  New compiler code introduces no ambient config and retains local Rust checks.
- **o11y-otel:** Applies to runner lifecycle signals, not deterministic compiler
  operations. No service sidecar/SLO or deployed file rotation applies. Outside
  actual spans omit trace/span IDs. Receiver mapping and all-sink privacy/failure
  controls must pass before calling the optional export integration verified.
- **formal-methods:** Executable analysis for selected finite composition/disclosure
  properties; precise specification for activation/custody and host assumptions.
  TD-009 records state, safety, bounds and exclusions. Risk is unsafe promotion of
  simulated policy truth/native custody; alternative correspondence is authored
  oracles and actual Rust controls. Larger/native/liveness proof remains open.

## Area Labels

`area:model`, `area:frontend`, `area:backend`, `area:bindings`, `area:conformance`.

## Concern Conflicts

Universal dialect versus native limits: define portable logical semantics and
refuse unsupported targets. Reuse UMF versus native embedding: share versioned
schemas/fixtures and validation results, not a TypeScript subprocess.
