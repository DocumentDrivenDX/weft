# Final qualified native Python and browser hosts

Fresh public Rust runtime, native ABI3 Python and real Chromium WASM builds use
both `truss-postgresql-qualified` and `ashlar-databricks-qualified` features. All
2,181 requests match complete native-qualified artifacts; no candidate opt-in or
metadata normalization is used. Python and browser return byte-identical output
to the fresh public Rust runtime, with 4,362 audited case/request/runtime joins.

CPython 3.12.14 on macOS ARM64 loads the actual ABI3 extension with subprocess
entry points disabled and PATH empty. Chromium 148.0.7778.96 uses Playwright
1.62.1, the actual new WASM and built transport, without Node globals or compiler
network access. The browser transport also normalizes a real WASM unreachable
trap and retires that instance. Both hosts reject invalid transport/surrogates.

Seven raw duplicate-key/malformed/resource cases pass through each actual host,
with atomic no-artifact refusals and exact public Rust parity. Eight semantic
receipt corruption controls refuse after custody rehashing. Complete original
cases/reports are compressed, with exact uncompressed byte/hash custody. Package
RECORD/native payload checks confirm the five-member wheel, exact loaded extension,
and no JavaScript/WASM sidecar. The Python initializer's first test expectation
was incomplete; its retained failed log is not a changed payload or compiler bug.

These are actual compiler embeddings, not additional database executions. Native
SQL semantics remain joined to the independently reconciled engine/profile
receipts. No broad interpreter/platform, distribution, signing or production data
authorization claim is made. Current workspace/30-criterion acceptance and PR #9
review/merge remain pending.
