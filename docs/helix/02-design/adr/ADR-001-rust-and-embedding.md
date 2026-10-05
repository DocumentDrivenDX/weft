---
ddx:
  id: ADR-001
  type: adr
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.architecture
      kind: informed_by
---

# ADR-001: Rust core with native Python and browser bindings

**Status:** Proposed. **Date:** 2026-10-05. **Decider:** Project owner.

## Context

The compiler must embed directly in Python and TypeScript/browser applications.
The previous TypeScript experiment proved a bounded compilation path, but Python
would require a JavaScript subprocess/runtime or service. UMF's implementation
language and Truss's accepted TypeScript decision do not govern this separate tool.

## Decision

Propose one pure Rust compiler core, PyO3/maturin native Python packaging, and
WebAssembly with a thin TypeScript wrapper. Treat direct native Python calls and
real-browser WASM calls as mandatory acceptance, not future wrapper promises.
Pin toolchain/dependencies only after SPIKE-001 proves the portable boundary.

## Alternatives

| Option | Benefit | Cost / disposition |
| --- | --- | --- |
| Rust with thin bindings | One implementation in-process across hosts | Rust UMF interpretation and platform packaging need conformance; proposed |
| TypeScript with sidecar/service | Reuses UMF TypeScript directly | JavaScript runtime/process boundary in Python; rejected for native Python requirement |
| Separate Python/TypeScript compilers | Native APIs in each language | Duplicated semantics and drift; rejected |

## Consequences and Risks

Weft consumes versioned UMF JSON/schema contracts without invoking TypeScript at
runtime. Its Rust reader needs independent agreement with UMF; no complete UMF
implementation is required. Wheels/WASM need reproducible platform builds.
Bun is suitable for documentation/JS wrapper tooling, not the Python runtime.

## Validation and Reconsideration

SPIKE-001 must run one compile/refusal case through Rust, native Python and a real
browser, retain unknown content/exact numerics and show no host-only imports in
WASM. Reconsider dependencies or packaging if a required host cannot run the
same core; do not weaken FR-10 silently. Versions/OS/browser matrix remain open.

## References

[Research](../../00-discover/research.md), [PRD](../../01-frame/prd.md),
[architecture](../architecture.md), PyO3 and wasm-bindgen official guides.
