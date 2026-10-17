# Exact non-temporal raw aggregates — 2026-10-16

WR-A10 / S11-S12 compiler candidate for PostgreSQL18 and Lakebase18. DSL
`avg_(c, scale=n)` renders the distinct closed `AVG_SCALE(column, scale_literal)`
with bounded scale 0..1000. The incoming proposed grammar is updated explicitly;
CONTRACT-071's typed API and exact half-even semantics remain unchanged. Existing
reviewed SQL `AVG(column)` retains native PostgreSQL semantics and does not
qualify DSL AVG.

AVG_SCALE uses numeric SUM/count, an exact power-of-ten multiplication, integer
quotient and remainder, then chooses the even neighbor on exact half ties for
both signs. Exact decimal multiplication restores the requested scale, and round
fixes its representation; no rounded AVG or fractional division intermediate is
used. Zero non-null inputs return NULL. PostgreSQL documents its [numeric div/mod
primitives](https://www.postgresql.org/docs/18/functions-math.html).

COUNT(DISTINCT column) is exact, ignores nulls, and returns non-null Integer.
SUM uses a numeric input accumulator even for integer columns, retaining Integer
output semantics beyond signed64-bit; decimal sums retain Decimal. MIN/MAX and
all owned string columns explicitly use C collation so comparisons, grouping and
exact distinctness cannot inherit a nondeterministic database collation.

BUCKET_WIDTH(column, positive_numeric_literal) uses exact numeric integer
quotient/remainder and a negative-remainder floor correction. Integer field plus
integer width yields Integer; otherwise Decimal. Nullability follows the field.
Literal widths are finite positive plain integers/fixed decimals. Bound, zero,
negative, null and exponential widths refuse; a bound-width domain/host profile
is not invented by this compiler increment.

Group custody compares complete compiled expressions, preventing a bucket group
from admitting its ungrouped raw field in projection, HAVING or ordering. ORDER BY
may name a projected alias; complete group expressions append ascending/null-last
as deterministic tie keys. Existing read-only/source/binding/native obligations
remain retained. This adds no native transaction or authority.

Rust1.90.0: eleven raw-read tests pass, no skips. Tests cover the closed aggregate
shapes/carriers, old AVG separation, scales0/2/38/1000, invalid scales/functions,
exact bucket group identity and ordering, and all prior prefix/null-safe/read
controls. The first broader run had two assertion failures: expected string SQL
omitted the newly explicit C collation, and a new bucket assertion assumed1000
instead of its fixture's actual rowCap10. Both assertions were corrected precisely;
no semantic refusal or runtime check was weakened. Logs retain the failures.
All11 Cargo/module boundaries and16 boundary tests pass; pinned specs pass.

Paired generated SDK/native positive/negative half ties, arbitrary integer sums,
decimal scale, all-null/empty sets, exact distinctness and signed bucket results
are still required. These compiler tests do not qualify native results. No date
buckets, approximation algorithms, managed engine or temporal qualification is
claimed. SDK temporal remains **HELD**.
