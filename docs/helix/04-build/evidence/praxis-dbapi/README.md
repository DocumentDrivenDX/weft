# Praxis DBAPI transport candidate

This extends the raw-read contract's compiler-owned transport. The original
PostgreSQL SQL is retained. A pinned Rust PostgreSQL tokenizer identifies only
numeric placeholder spans; original bytes, including Unicode and quoted content,
are preserved. DBAPI percent escaping and occurrence order belong to Weft, not
Praxis or a host SQL rewriter. Related target fragments defer transport until
source/target parameter inventories are composed. Both SQL representations and
orders are sealed in the original structural digest.

The local installed0.4.46 candidate wheel passes187 native PostgreSQL18.6
assertions with UMF's actual generated client, Psycopg3, Psycopg2 autocommit on/off
and SQLAlchemy Core sync/async. They include bound values containing quotes,
percent/dollar characters, backslashes and Unicode, authenticated person/batch
reads and parameterized keyset continuations. A separate compiled literal query
retains `雪50% $1` in SQL and returns the same result through original `$n` and
DBAPI transport. The exact wheel/test/source hashes, versions and all assertions
are in [the receipt](receipt.json). This is an unpublished source candidate.

Four tokenizer controls pass literal/identifier/comment/dollar-quote preservation,
repeated/out-of-order slots, Unicode/CRLF/tab spans, percent-only SQL and malformed
slots/quotes. Four related-read regressions pass. The broader library replay
passes183 tests with no exclusions, including all434 precision/scale decimal
fixtures (497.79 seconds for the full run). The initial
run exposed a real related-slot composition defect, now fixed, and two historical
hash assertions affected by the additive transport metadata. The raw-bulk golden
is explicitly updated. The twelve-case public-filter fixture retains prior
baseline hashes and adds separate DBAPI successor hashes; comparison against the
cached baseline wheel changes only the two transport fields and resulting raw/
nested/outer structural digests. Original request bytes/hashes are retained.
Cargo module boundaries still pass with eleven workspace modules and no new
crate dependency. No test/global qualification rule is disabled.

The native executor is a bounded UMF fixture, not production catalog/privilege
admission. Direct native writes qualify session ownership only; they are not
Weft action qualification. DDL, policy/grant generation, digest migrations,
complete table commands/replay, managed acceptance and immutable prereleases
remain open under the full Praxis release goal. No native or language guarantee
is inferred for targets outside this PostgreSQL18/Lakebase18 compilation subset.


The exact-source Linux native job also passes187 assertions in
[UMF run39258945575](https://github.com/DocumentDrivenDX/umf/actions/runs/39258945575).
Its installed compiler wheel hash is retained in the receipt, separately from
the local macOS wheel. Full hosted Weft conformance was dispatched as
[run39259830007](https://github.com/DocumentDrivenDX/weft/actions/runs/39259830007);
its completion remains a separate gate. No immutable prerelease is published.

The first hosted conformance run failed before compiler tests because the new
private `dbapi.rs` file was absent from the stricter Rust source/public-surface
inventory. The actual diff is exactly one private file, no new public declaration
or crate edge. Its ownership is now recorded explicitly in the PostgreSQL backend
map. The complete `modules:check` command, including JavaScript/Rust inventories
and all negative controls, passes locally; the checker itself is unchanged.
This corrects an omitted ownership declaration, not a widened dependency rule.

The successor hosted run39263110193 passed those ownership gates, then refused
the source-mutation gate because its resource panic signatures still named line
numbers preceding the SPDX header. Resource assertions now carry explicit
messages; detection requires the original test file, named assertion, native
Object/Array value category and exactly one failing selected test. New negative
controls reject another assertion, value or source file while tolerating line
movement. Normal and optimized configuration suites each pass16 tests.
[Actual optimized source replay](resource-mutations.json) passes all7 original
mutants with clean baselines and retained original/mutant digests. It does not
claim diagnostics-export or complete hosted conformance qualification.
