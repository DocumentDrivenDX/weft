# Application-read frontend conformance

Run `sh scripts/run-b002a.sh` with the pinned toolchain and Chromium profile in
[B-002A evidence](../../docs/helix/04-build/evidence/B-002A-application-reads.md).
It preserves the original gate and adds 620 application frontend cases, an
independent 18-scenario result oracle, versioned schema checks and native/browser
byte parity. `cargo test -p weft-core` runs the native tests alone.

`generate.py` reproducibly serializes independently authored acceptance/refusal
cases; expected decisions never come from compiler output. Its 575 numeric
parameter boundaries retain the existing independent corpus expectations.
`generate_schemas.py` reproducibly writes the draft structural schemas.
`reports.py` collects compiler reports without inventing expected results.
`oracle.py` interprets typed plans with bags, Python integers and Decimal, compares
hand-authored results and deliberately retains duplicate fixture edges.

This evidence qualifies the frontend component. Public Python/compile artifacts,
native storage decoding, explicit-null permission and production database/layout
support require the later implementation gates. Descriptors retain ideal
availability separately from type and native representation. Relationship
multiplicities/lifecycle retain original orientation with an inverse flag; source
and target keys follow the traversal direction. No property join creates a
relationship.
