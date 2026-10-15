# Browser-local Truss selection lowering

The exact `truss-browser-binding/0.1` profile describes PGlite0.5.10,
PostgreSQL18.6, original Truss layout SHA c203d0, store correspondence,
accepted document/model pin, catalog revision and bounded scan size. It admits
one mapped Record with complete ordered required String member mappings and
its original single-member primary key. Native65/66 binding validation remains
unchanged; no browser metadata is rewritten as a native release or credential.

`truss-browser-local` on the production WASM crate adds
`MutationCompiler.register_browser_selection(configurationJson)` and
`lower_selection_json(handle,requestJson)`. Registration requires original
model/modelPin/binding Artifacts; lowering requires unchanged registered bytes.
The shared Rust renderer parameterizes predicates and projects native identity,
version and original key values. Browser preflights validate the entire target
population's scan size, identity/version, every Record member (including unused
fields), and duplicate keys before selection filtering or LIMIT.

The basic Chromium153 case builds a real catalog using Truss's original
registerCatalog and the unchanged owner-authored Products source. Original
layout, lineage, containers, plan and scalar validator SQL execute in PGlite.
Two rows are seeded administratively; all four complete-population preflights
pass and the original Weft selection returns bar, native identity1/version1.
The receipt retains exact WASM and Truss input hashes. The focused Rust test
checks local mapping/key correspondence and complete member preflights; the
existing native65/66 lowering regression also passes. Browser member predicates
are combined in one bounded preflight, preserving the full admitted64-member
subset under the outer registry collection bound; the focused test exercises
that synthetic64-member mapping through the original outer registry.

Reproduce after building weft-wasm with `--features truss-browser-local` and
wasm-bindgen0.2.105 `--target web`:

```sh
cargo test -p weft-postgresql --lib truss_mutation
bun tests/mutation/local-binding-browser-check.ts /path/to/playwright/package.json /absolute/wasm-web-output /absolute/truss-checkout /absolute/receipt.json
```

The original scalars.sql dependency is explicitly installed in this component
case. The browser provider must install/verify it before enabling selection.
Serialized store metadata is correspondence only. Issued owner/read grant,
current source/cut checks, public mutation, sealing, effects, persistence/replay
and recovery are not supplied or qualified by this compiler checkpoint. The
host must hold and independently check those capabilities before executing SQL.

The64-member regression fixture is separately authored synthetic input, prepared
through original UMF public prepareActionPlanModel (producer633f8), retaining
model8e83af6f and preparation3dba9861. Regenerate with
`bun tests/mutation/generate-local-wide-fixture.ts /absolute/umf-source-root`.
It supports this mapping limit check, not a new qualified adopter model.
