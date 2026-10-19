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
passes182 tests with one decimal-domain test excluded: that test remains running
in the original full run, so full-library completion is not claimed. The initial
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
