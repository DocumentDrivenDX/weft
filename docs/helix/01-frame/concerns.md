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

## Project Overrides

Rust fills the language-runtime slot for the compiler, rather than HELIX's shipped
TypeScript/Bun default (ADR-001). Bun may run documentation and JavaScript wrapper
tooling; it is never required by Python consumers. No datastore/deploy/auth/UI slot
applies to a pure compiler library. These selections are proposed, not evidence
of a working Rust implementation; the embedding gate can revise tool choices.

## Area Labels

`area:model`, `area:frontend`, `area:backend`, `area:bindings`, `area:conformance`.

## Concern Conflicts

Universal dialect versus native limits: define portable logical semantics and
refuse unsupported targets. Reuse UMF versus native embedding: share versioned
schemas/fixtures and validation results, not a TypeScript subprocess.
