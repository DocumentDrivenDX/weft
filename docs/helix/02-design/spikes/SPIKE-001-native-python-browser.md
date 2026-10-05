---
ddx:
  id: SPIKE-001
  type: tech-spike
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.architecture
      kind: informed_by
    - id: ADR-001
      kind: informed_by
---

# SPIKE-001: Shared Rust/Python/browser compiler boundary

## Question and Scope

Can one pure Rust model/compile boundary run in native Python and browser WASM
without a JavaScript sidecar, exactness loss or host I/O? Bound the investigation
to 2 engineering days and one synthetic core 0.7.0 model; no production backend.

## Experiment

Evaluate sqlparser-rs as a parser component under Weft's independent grammar gate.
Pin Rust, parser, PyO3, maturin, wasm-bindgen and wrapper tooling versions. Import
a local Python extension; instantiate WASM in real Chromium. Use identical model
bytes, pins and query; compare canonical reports and refusal codes across hosts.

## Pass / Fail

Pass requires direct Python import without Node/Bun/server; same core build
provenance; exact large integer/decimal strings; unchanged unknown extensions;
known limit/digest/unsupported-feature refusals; no host globals/network in the
browser. Record memory/artifact sizes and packaging friction, without asserting
performance targets from one sample. Failure retains inputs/logs and revises
ADR-001/tool choices before compiler implementation proceeds.

## Output and Decision

Output is an execution evidence record with versions, bytes/hashes, commands,
observations and limitations. The bootstrap had no evidence. B-001 execution is now recorded in [the evidence report](../../04-build/evidence/B-001-native-python-browser.md); its tested embedding gate passes with the stated subset and platform limitations. Product language
and backend contracts remain authoritative regardless of parser acceptance.
