# B-001 execution evidence — 2026-10-05

**Outcome: pass for the bounded spike.** One Rust core ran in a native Python extension and real Chromium WASM, producing byte-identical reports for **597 scenarios**. Proceed with Rust/PyO3/WASM for B-002. ADR-001 records that engineering direction; public contracts, full release qualification and backend layouts remain unfinished.

## Profile and provenance

| Component | Tested version/profile |
|---|---|
| Host | macOS 27.0.1, arm64 |
| Rust | 1.90.0, aarch64-apple-darwin and wasm32-unknown-unknown |
| Parser | sqlparser-rs 0.63.0, std feature; separate narrow grammar gate |
| Python | CPython 3.12.14; local cp39-abi3 macOS arm64 wheel |
| PyO3 / maturin | 0.27.1 / 1.9.6 |
| wasm-bindgen crate/CLI | 0.2.105 / 0.2.105 |
| Browser / harness | Chromium headless 153.0.8010.12 / Playwright 1.62.1 |
| Host tooling | Node 24.19.0, Bun 1.4.2; neither is required by native Python calls |

Cargo.lock pins transitive Rust dependencies; bun.lock pins documentation/browser tooling. Each successful report contains matching core-source and Cargo.lock SHA-256 values in all three runtimes. This establishes common source/dependency provenance, not identical native/WASM machine code. [Artifact hashes](b001/artifacts.json), [native summary](b001/native-summary.json), [browser summary](b001/browser-summary.json) and [execution log](b001/run.log) retain measured evidence. Build output is ignored and not published as a supported package.

## Observations

- Three Rust integration tests passed; the numeric test independently asserts all 575 committed numeric scenarios. The separate native harness adds 22 boundary cases, then compares all 597 against Rust CLI output. Chromium compares all 597 responses byte-for-byte with the same Python reports.
- Exact unsigned 64-bit maximum `18446744073709551615` and decimal `9007199254740993.12` survive as text. Unknown extension content, including a 40-digit unquoted JSON integer, survives through unchanged owning-document source text. No Python or JS numeric conversion occurs at the string boundary.
- Pin, binding digest, version, selected type/facet, alias/name, SQL/request size, malformed JSON/SQL, duplicate-key and excluded syntax refusals are explicit and atomic for the tested cases. Hostile quoted text remains parameter data.
- A separate direct Python import/call succeeds with no executables on PATH and subprocess.Popen disabled. The extension's source is a thin call to the Rust core; no JS runtime/server is involved. The Rust CLI subprocesses belong only to the parity test harness.
- Browser resources initialize through local test routes. During compilation, fetch/XMLHttpRequest/WebSocket/Worker throw if invoked; no external requests occur and Node process/require globals are absent. The generated WASM imports only `wbg.__wbindgen_init_externref_table`, a reference-table initializer; no WASI, filesystem or network interface is imported.
- Native extension: 6,298,112 bytes. WASM: 5,270,652 bytes. Generated JS glue: 6,420 bytes. WASM linear memory is 1,441,792 bytes initially and 1,769,472 bytes after the suite. These are observed linear-memory sizes, not process memory, peak allocation or performance targets. The complete parser contributes packaging weight that B-002 should evaluate.

## Corrections and packaging friction

An initial numeric run exposed rejection of insignificant trailing decimal zeros. The implementation was corrected and the unchanged expected corpus then passed. The initially assumed wasm-bindgen import namespace was corrected to the generated `wbg` initializer and kept as an exact allowlist. Bundled Playwright's expected browser was absent; the installed Chromium executable was selected explicitly and its exact version recorded. Rust, maturin and wasm-bindgen were installed in isolated temporary tooling directories; no global PATH was modified.

## Reproduction and limits

Use [the spike instructions](../../../../spikes/b001/README.md) and `sh scripts/run-b001.sh`; the successful run used absolute WEFT_PYTHON and WEFT_CHROMIUM_EXECUTABLE overrides. The captured log predates the harness's final exact package-version assertion; the browser summary comes from a final passing rerun of that assertion with unchanged compiler artifacts.

The public compile envelope/IR, full UMF validation, join/group/SUM execution, module bundles, third-backend registration, production Truss/Ashlar mappings, database execution, effective authorization, publication and broad Python/OS/browser matrices are not established. The spike emits explicitly synthetic candidate SQL and a separate `weft-spike/0.1.0` report. The cp39 ABI tag is a packaging declaration, not evidence that Python 3.9 ran. All 24 public story acceptance criteria remain planned; this bounded evidence does not mark them covered.
