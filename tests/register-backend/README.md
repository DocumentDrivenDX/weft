# Registered backend component conformance

Run `sh scripts/run-b003.sh` with the toolchain/Chromium profile in
[B-003 evidence](../../docs/helix/04-build/evidence/B-003-backend-interface.md).
It includes the previous frontend gates, backend tests, independent SQLite
execution, manifest schema checks and real Chromium/native byte parity.

The shared test-only `fixture.rs` implements the generic associated-type backend
trait. `pipeline.rs` tests registry and atomic failures; `probe/` links that same
plugin for native/browser reports. `generate.py` writes independently authored
26-case decisions and rows; `oracle.py` executes seven emitted SQL cases using
host SQLite and verifies 19 refusals. Expected results never come from emitted
SQL. Candidate declarations remain explicit and default-refused.

The fixture profile supports a single required string projection with ASCII
physical identifiers. Its declared evidence IDs are component references. SQLite
execution does not qualify any production SQLite, PostgreSQL or Databricks profile.
Public compile wrappers, native layout/decoding/publication and distribution
remain later gates. The compiler does not execute SQL or load code from content.
