# Large original Truss catalog binding

`genuine-thousand-type-request.json.gz` contains the exact request captured from
Truss's installed public query API against runtime source
`81940ff251807934c2730535744e08b1d3df66a8`, native manifest
`b0f338e8d404b53b91e852c2efca04d761c0ecf2b2715852ee0e5ab8c69cb919`.
The generated `many-types` schema has 1,000 records with twelve fields each.
The fixture registered it and committed 100 objects and 100 relationships, then
requested `SELECT x.* FROM "R0" x LIMIT 51` with complete-entity semantics. The
fixture database was disposable; its opaque installation identity grants no
native authority. This file qualifies pure compilation, not database execution.

The unchanged source has 2,528,638 UTF-8 bytes / 179,012 JSON values; the original
binding has 2,025,776 bytes / 110,021 values. Published Weft 0.4.41 refused the
binding with `WFT-LIMIT` despite its existing four-MiB byte envelope. The scoped
runtime reader admits this input while retaining byte/depth/value limits,
duplicate-member checks and exact source pins. Generic readers remain unchanged.

`wheel-smoke.py` exercises the actual installed wheel, requires successful
compilation and original binding/model pins, and checks the old registration,
duplicate members, node overflow and byte overflow. Every release-wheel platform
runs it. Rust unit tests separately exercise exact parser boundaries. The
coordinate index preserves original array order and source values; the complete
2,424,290-byte compiler artifact matched the previous linear-lookup implementation
exactly in the local comparison. Compiler-only timings were 2.317 seconds before
indexing and 0.465 seconds afterward on macOS ARM64 with Rust 1.90.0 release mode.
These single local observations are not latency guarantees.
