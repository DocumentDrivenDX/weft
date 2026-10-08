# B-005 Truss/PostgreSQL integration

This directory defines independent native probes before backend implementation.
The current owner source is Truss's **draft layout 0.2** at
`/private/tmp/claude-501/truss-spec-wt`, not an accepted Weft binding or production
qualification. Do not substitute the earlier partitioned spike layout.

`native-boundaries.sql` is a read-only engine semantics probe over VALUES, with
no Truss deployment or schema mutation. It checks exact numeric extraction/SUM,
UTF-8 C-collation ordering, trailing spaces, JSON absence/null and numeric cursor
ordering. It is not an oracle derived from compiler SQL and does not certify the
adapter or catalog mapping. `check-native-boundaries.py` validates its independently
specified output captured using `psql -X -A -t -f native-boundaries.sql`.

Full B-005 also requires all applicable original and application-read corpus
queries against owner-approved tables/bindings; actual prepared statement/typed
parameter execution; document/module identity and catalog revision guards;
whole-value exact codecs; forward/inverse relationship keys; role isolation and
read context obligations; complete Rust/Python/browser parity for the linked
adapter. An unavailable server or missing storage-owner decision is not a pass.

Pure PostgreSQL emission primitives are implemented in `crates/weft-postgresql`.
`cargo test -p weft-postgresql --locked` passes quoting/boundary and exact-slot
checks. This crate performs no IO and has no storage registry entry yet. Identifier
length assumes the standard 63 UTF-8 byte server profile; native qualification
must verify this setting rather than truncating names. Slot lexical/domain checks
remain at the common backend emission boundary before a compiled artifact.


On 2026-10-06 the nine native engine boundaries pass on PostgreSQL 17.9.
`prepared-check.py` also checks four generated exact slots/quoted labels using
SQL PREPARE. This does not qualify host protocol binding or a Truss adapter.
See the B-005 preparation evidence for exact versions, hashes and remaining gates.


Candidate development has resumed on explicit owner direction. The frozen proposal
schema closure is compiled into `binding.rs`; it performs no IO and distinguishes
byte/shape admission from source/native qualification. `native-homes.py` exercises
scalar access templates across JSONB and typed row columns, preserving exact values
and duplicates and detecting a broken payload. These are access-layer witnesses,
not yet full registered Weft query compilation or installed-layout qualification.

The candidate now registers through the actual core Backend trait. Its current
0.1 relational subset passes 26 native query/result comparisons (13 original
independent queries across both homes). `compiler-native.py` checks exact bags,
empty aggregates and text distinctions. Application 0.2 and production enforcement
are still being implemented; the profile retains candidate status and host obligations.
