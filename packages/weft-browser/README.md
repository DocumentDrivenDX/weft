# Weft browser transport

Build from the repository root with `bun install --frozen-lockfile` followed by
`bun run browser:build`. TypeScript 5.9.3 emits `dist/index.js` and
`dist/index.d.ts`; the package exports those built files. Generated output is
ignored by Git and must be built before packaging. The package remains private.

The host builds/selects a trusted Weft WASM compiler with the required backend
features, initializes its wasm-bindgen module, then calls `bindCompiler(module)`.
The transport does not bundle or download a compiler, connect to databases or
implement SQL/model semantics. `compileJson` passes scalar strings to Rust.
A WASM runtime trap retires that instance; initialize a fresh module to recover.

Distribution requires the owner license decision and final release evidence.
