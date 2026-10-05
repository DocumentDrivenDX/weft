# B-001: native Python and browser embedding spike

This experiment implements one shared Rust core, a PyO3 extension and a wasm-bindgen browser export. It is **not** the public Weft compiler. Its output uses `weft-spike/0.1.0`, not CONTRACT-003's compilation artifact. Synthetic SQL is inert illustrative output; no database result or production backend compatibility is asserted.

Slice: one supplied UMF 0.7.0 owning document and selected module; one explicit source alias; one qualified column projection; optional WHERE equality-to-literal predicates combined by AND. Known required single fields: strings, booleans, integer widths 1–64 and decimal precision 1–28. The core gates sqlparser's broader syntax with a separate narrow grammar. Models retain their exact source text and unknown numbers/extensions; selected unsupported fields/facets refuse. Digests, SQL/request/model/token/JSON depth/value limits and duplicate keys have guards. This reader is a participating-subset experiment, **not complete UMF schema or semantic validation**. No joins, aggregates, module bundles, backend registry, public IR, complete diagnostics, policy obligations or production binding validation are implemented.

`check_native.py` adapts only the 575 numeric scenarios from the independent specification corpus to `spike.synthetic`; the original fixture files and expected values stay unchanged. It adds 22 spike scenarios for exact-value transport, unknown numbers, hostile strings, pin/version/type/grammar/limit/duplicate-key guards and atomic refusal. Reports compare byte-for-byte across Rust CLI, native Python and real Chromium. The Python-only smoke disables subprocesses and clears executable PATH. Browser initialization uses local routed resources; compilation runs with network APIs disabled and no Node globals. WASM imports are checked against the single reference-table initializer.

## Reproduce

Install Rust through rustup (the repository pins 1.90.0 plus wasm32); add rustfmt. Use a Python 3.9+ virtual environment and install `maturin==1.9.6`. Install `wasm-bindgen-cli` **0.2.105** with `cargo install wasm-bindgen-cli --version 0.2.105 --locked`. Run `bun install --frozen-lockfile` and `bunx playwright install chromium`. Put `cargo`, `rustc`, `wasm-bindgen` and `node` on PATH. Set `WEFT_PYTHON` to the virtual environment's **absolute** Python path, then run:

```sh
sh scripts/run-b001.sh
```

`WEFT_CHROMIUM_EXECUTABLE` may select an existing Chromium executable. `WEFT_PLAYWRIGHT_MODULE` may point to an installed Playwright ES module; it must be 1.62.1 for the recorded harness profile. The harness verifies the actual package version. Generated wheels, WASM, case payloads and runtime summaries stay under ignored `target/`. The core and Cargo.lock hashes embedded in each compiled report establish shared source/dependency provenance, not identical machine-code binaries.

See the [execution evidence](../../docs/helix/04-build/evidence/B-001-native-python-browser.md) for the tested platform, versions, artifact sizes, memory observations and limitations. Full release conformance and broad wheel compatibility remain B-002 onward.
