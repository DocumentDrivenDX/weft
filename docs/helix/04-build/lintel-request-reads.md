# Lintel connection compiler: local evidence

The additive `PreparedTableOperation.bind_connection_json` is implemented by the
same Rust PostgreSQL backend and forwarded by the runtime and Python binding.
The original closed invocation shape and UMF table declarations are reused.
Ordinary bind output remains unchanged. Connections refuse non-page operations
and nullable order fields; zero-size binding is specific to this entrypoint.

The compiler emits an authenticated-position-compatible page program and a
parameterized previous-position EXISTS probe, including DBAPI occurrence order.
It does not execute SQL or grant authorization. The host must use one qualified
read view and preserve current admission independently.

[Verification](evidence/lintel-request-reads/verification.json) records20 passing
compiler regression tests and installed PostgreSQL18.6 execution through UMF's
new request scope, including policy-filtered flags, deleted positions, interior
and terminal continuations, first=0 and resized pages. Native Rust and Python
share this implementation. No new WASM/browser surface or managed target is
qualified by this local test.

The candidate wheel inherits source version0.4.48; its SHA-256 and exact source
hashes define this local build. Nothing has been released. Cargo dependency
boundaries pass unchanged (11 workspace modules). No new dependency is introduced.
