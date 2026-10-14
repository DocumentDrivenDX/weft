# Weft portable mutation compiler implementation evidence

Scope: core0.8.0/actions0.1.0, plan1, weft-mutation0.1.0 and weft.update-selection1. This starts US-008/009 after UMF US-902/903. Both stories remain incomplete; planned ACs and native gates are not upgraded by this component milestone.

`MutationRegistry` in the original Rust core consumes explicitly installed original UMF document/action/preparation artifacts. The registry checks exact byte hashes, original compiled preparation schema and producer pin, complete unique bindings and finite metadata. Requests cannot install action or backend code. The separate UPDATE parser and resolver use the existing logical catalog and original exact literal domains. Compilation emits inert source/parameter/selection/plan artifacts and binding provenance; it neither discovers data nor queues or executes work. Source SET order is preserved while UMF inspection matches the complete unique binding set against original action effect order.

Python `MutationCompiler` and WASM `MutationCompiler` call this same Rust implementation. The browser wrapper only validates scalar transport strings and binds an explicitly initialized module. The CLI example is host-only fixture I/O. No JavaScript compiler runs inside Python and no native I/O/credential boundary is introduced into the compiler.

Verification:

- Cargo1.90 `test -p weft-core --test mutation_compile --locked --offline`:8 tests pass, including the independent original UMF canonical/native-SHA full-response golden.
- Fresh maturin native wheel and wasm32 debug build + wasm-bindgen web glue succeed. Both were rebuilt after the final iterative parser fix.
- Native Python `tests/mutation/python-check.py`:32 independently expected status/golden cases, exact artifact hashes and a separate authored row-selection oracle pass. The small evaluator is limited evidence, not full backend qualification.
- Actual Chromium153 via `tests/mutation/browser-check.ts`:all32 full response strings equal native Python, independent WebCrypto hashes pass, unpaired surrogate transport refuses, Node globals absent, no external requests. See retained JSON receipts and source/artifact digest inventory.
- Astra Ultra independently reviewed the final code and boundary artifacts; see `astra-review.md`.
- Broad core regression reaches two backend-registry expectation failures. This is not recorded as an all-green baseline; its separate diagnosis is retained below.

Reproduction uses the existing offline toolchain under /private/tmp/weft-toolchain: CARGO_HOME=cargo, RUSTC=rustup/toolchains/1.90.0-aarch64-apple-darwin/bin/rustc; maturin runs with that toolchain bin first in PATH. Run installed native Python first to write target/mutation/parity-cases.json, then Bun browser-check.ts with WEFT_PLAYWRIGHT_MODULE pointing to the installed Playwright module. No executable location is taken from model/plan content. Build products remain in target; retained evidence is under docs/helix/04-build.

Remaining implementation: backend registration/admission/lowering and native preflight, exact identity/version decoding and numeric ordering with maxItems+1; broader independent scalar/operator/model boundary controls; a second registered test consumer; read binding compatibility; Truss frozen materialization, protected native batch execution/replay/recovery, and the UMF website sample-data demonstration. No Truss, Ashlar or whole-slice support is established here.

### Existing regression discrepancy

The two backend-registry failures reproduce in a scratch copy with all three new mutation modules excluded from weft-core lib.rs. The source/user changes in the shared checkout were not reverted. `capability_declaration_guards_refuse_at_their_exact_phase` expects WFT-CAPABILITY but receives WFT-BACKEND-VERSION for logicalDomain; `target_profiles_require_each_identity_and_object_settings` expects the older profile diagnostic but receives Opaque declaration member must be an object. See backend-registry-without-mutation.log. They are preexisting relative to this mutation-module addition, and remain unresolved; no broad-green regression claim is made. Initial scratch attempts failed for omitted workspace/schema files before the final complete baseline probe reproduced both assertions.

The remaining selected read/qualification targets pass after the broad run stopped at registry failures: compile-envelope6, qualification-properties7, qualification-resources5 (18 total). The exact integer/decimal property suite took130seconds, with no latency guarantee. Both repositories pass git diff --check.
