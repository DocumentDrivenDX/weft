# Browser local UPDATE compilation

UMF CONTRACT-905 at40f011402bf8f0c9512225975b62a034f1da325c supplies the
unchanged original local-products-input fixture (SHA256
2a17a1dc0df402ff6ea4fe10c14314e5f928951a70294e32734d04746616de8e),
including sourceb8ce and preparation6f5e. The registry pairs local-tx/1 only
with original producer633f8; retained native tx/1 still requires bf743.
Original schema bytes are identical and reused, with separate owner provenance.
The uniquely qualified source action must declare the complete local actor and
policy bundle, with resources equal to read then write frames in declaration order.

The nine existing/focused Rust compiler tests pass, including original native
plan golden bytes, local UPDATE compilation and crossed producer refusals. Rewriting the local preparation and pin to
native tx/1 still refuses against the unchanged original local source; native
source admission rejects either recognized local actor or policy declaration.
The production WASM crate now exports MutationCompiler over the same Rust
registry. Actual Chromium153 compiles the original local Products UPDATE and
checks its plan digest with WebCrypto. The retained receipt identifies the
actual release WASM artifact and original plan digest.

Reproduce with the pinned Rust toolchain and wasm-bindgen0.2.105:

```sh
cargo test -p weft-core --test mutation_compile
cargo build -p weft-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir /absolute/web-output target/wasm32-unknown-unknown/release/weft_wasm.wasm
bun tests/mutation/local-browser-check.ts /path/to/playwright/package.json /absolute/web-output /absolute/receipt.json
```

Compilation issues no authority. Local physical binding/lowering, issued owner
and read grant, sealing, effects and replay remain unfinished. The native
runtime binding and native security profiles are unchanged. This is source
component evidence, not a published wheel or completed browser UPDATE workflow.
