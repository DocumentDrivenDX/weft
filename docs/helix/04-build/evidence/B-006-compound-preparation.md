# B-006 compound preparation checkpoint

State: in progress, 2026-10-08. This is not story acceptance.

The pure adapter now admits an explicit candidate exact JSON compound encoding
and lowers finite UMF descriptor graphs into a recursive runtime-value walk.
Ordered JSON tokens preserve list order, map keys, structured members, numeric
string leaves and optional envelopes. Unknown reachable extension meaning and
missing/unknown encoding refuse. Native-null permission is absent. Runtime
depth/node guards and duplicate physical-ID guards precede user predicates.

The valid new compiler test failed at mapping admission before implementation;
17 Ashlar Rust tests now pass, including recursive/cyclic graphs and four
encoding/dependency refusals. All 276 previous full artifacts remain unchanged.
133 independently authored native cases compile before fixture creation. Their
native execution is running against one private immutable publication; the
complete result is not yet available.

Read-only prerequisite probes prove recursive VARIANT traversal, exact
container casts, explicit JSON-null distinction and ordered token assembly on
the existing warehouse. Correlated seeds are refused by the engine, so the
codec uses independent whole-owner CTEs. The first generated walk used an
incorrect explode struct reference; it was corrected and the native run resumed
read-only against the same publication. The quoted/backslash member-path probe
failed, so dictionary-based member lookup must replace that path construction
before general structured-name coverage is qualified. Depth/node boundaries,
compound embeddings and actual host enforcement remain unfinished.

Native recursive syntax and limits were checked against the official
[Databricks CTE reference](https://docs.databricks.com/aws/en/sql/language-manual/sql-ref-syntax-qry-select-cte),
last updated 2026-09-11. That reference does not identify this warehouse's
release/channel; native capabilities remain candidate. Probe statement receipts
are in B-006-recursion-probes/. No production compatibility is implied.
