# US-005 embedding acceptance review

Scope: observed macOS ARM64 CPython 3.12.14 and Chromium 148.0.7778.96,
with the explicitly named feature builds and hashes in the component receipts.
No wider interpreter/OS/architecture or distributed-package claim follows.

| Criterion | Evidence and finding |
| --- | --- |
| AC1 | Fresh public 1273-case and Truss 76-case native wheel runs, plus Ashlar 463-case fresh execution of retained wheel. Actual loaded extensions are hashed, PATH is empty and subprocess attempts refuse. Wheel content inspection confirms Mach-O native modules and no JS sidecar payload. |
| AC2 | Real Chromium public 1273, Truss 76 and Ashlar 463 response parity. Compiler execution blocks host I/O; recorded imports and runtime versions accompany WASM hashes. Initialization fetches trusted host assets before compilation; no content-selected executable URL is used. |
| AC3 | Independent metadata audit finds 24 wide-integer text/exact-integer artifacts, two nullable text/exact-decimal global SUM artifacts and 2526 string parameter slots in actual Ashlar Python reports. Full-byte parity carries these fields unchanged in browser/CLI. Native exact values and empty results remain separately proved by B-006 engine receipts. |
| AC4 | Python and WASM lib.rs entry points pass strings directly to weft_runtime::compile_json. TypeScript transport only validates scalar strings, delegates and retires a trapped instance; it does not resolve names/types or lower plans. Full-response parity covers diagnostics/pins/spans/qualification/obligations. Raw resource checks and real WASM trap controls supplement ordinary refusals. |

These are criterion-specific findings on the observed component matrix. The
full release acceptance and supported-native-profile audit remains open. The
public fixture matrix contains seven emitted successes and 1266 refusals;
its count must not be presented as native backend executions.
