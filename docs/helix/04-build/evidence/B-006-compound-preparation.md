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
133 independently authored native cases compile before fixture creation. All 133 completed against one private immutable publication: 56 exact results
and 77 refusals before the user query. The original complete receipts are in
B-006-compound-native-initial/. The dictionary-lookup revision also passed all 133 against that same immutable
publication. Its exact receipts are in B-006-compound-native/.

Read-only prerequisite probes prove recursive VARIANT traversal, exact
container casts, explicit JSON-null distinction and ordered token assembly on
the existing warehouse. Correlated seeds are refused by the engine, so the
codec uses independent whole-owner CTEs. The first generated walk used an
incorrect explode struct reference; it was corrected and the native run resumed
read-only against the same publication. The quoted/backslash member-path probe
failed. Exact dictionary lookup with explicitly binary string keys passed
both read-only native probes and now replaces member-path construction.
The separate read-only native controls now pass six cases: escaped/Unicode/case/
trailing-space member names, unknown members, exactly 100000 nodes, 100001-node
refusal, depth below 128 and depth-128 refusal. These use a synthetic owner relation;
they do not claim publication qualification. Complete receipts are in
B-006-compound-boundaries-native/. Fresh native Python and real Chromium/WASM builds match 415 full artifacts: the
276 previous cases, 133 compounds and six boundary controls. The Python extension
has SHA-256 ea82e542ff6a3f046b549519349a963a0f5681029c121c9b1a5c3b468895acb3;
the WASM has SHA-256 5549ad4be06a9ebea158f13eb32ce3f4fa224bd3ca4487ef783151cb8f960245.
Complete summaries are B-006-compound-{python,browser}-summary.json. Python runs
with PATH empty and subprocess creation forbidden; Chromium 148.0.7778.96 proves
byte parity without runtime IO or Node globals and exercises actual trap
retirement. Compound entity/keyset combinations and actual host enforcement
remain in progress. A test-only buffered host fixture passes 49 independent phase/
transport/unknown-meaning controls; its actual native transport driver is prepared
but has not completed. B-006-host-before.log retains the missing-helper baseline.

Native recursive syntax and limits were checked against the official
[Databricks CTE reference](https://docs.databricks.com/aws/en/sql/language-manual/sql-ref-syntax-qry-select-cte),
last updated 2026-09-11. That reference does not identify this warehouse's
release/channel; native capabilities remain candidate. Probe statement receipts
are in B-006-recursion-probes/. No production compatibility is implied.

## Final component checkpoint

The frozen CLI (SHA-256 02f6db7df59ce6c075995083a2ff4750545d89765df4a7af0d101864f74d562c)
passes 48 whole-entity/keyset combinations of required/optional nested, structured,
cyclic and mapped structured values, with props or native BIGINT ID homes.
B-006-compound-application-native/ contains the complete final receipts.

The actual native host driver passes 30 cases across scalar, optional, relationship,
entity, page and compound reads: six buffered results and 24 discarded buffers
after caller, policy, publication or retained-file custody changes. It verifies
actual caller identity, manifest/table UUIDs, version vectors, owner DDL schemas
and retained-version reads before guards and before publication. The independent
49-case host fixture covers additional pre-query, unknown-meaning and exact
transport refusals. Its authority/lifecycle/projection certificates are synthetic
fixture attestations, not a production policy service.

Earlier harness schema/0.1 descriptor mistakes and a concurrent Cargo rebuild
were rejected. No rejected run closes a gate. Native runs now use an immutable
CLI outside Cargo's target directory. Final accepted statement ranges and the
fixture hash are in custody.json; full earlier attempts remain distinguishable.
Public host receipts hash caller text while retaining exact equality and the
canonical original response digest; raw receipts remain local.

The presence declaration now explicitly includes scalar or compound envelopes.
This corrects metadata only. Historical native artifacts retain their original
declarations; python-check.py applies the independently specified single-field
correction and compares every other field exactly. The final native Python build
matches 463 full artifacts. Final Chromium matches all 463 artifacts with byte parity; its exact WASM
SHA-256 is 73cf013ac6b14e07222f81201820cc049975c04fa93e3db8f37813e0712bc046.
The fresh native Python extension SHA-256 is
8e04e6812141e895c8002624d0d81e9288dcc645d918855a2fecc5fd3546d831.
The final CLI matches that entire corpus as well. The declaration-only bridge
amends 172 historical capability declarations and leaves native SQL, parameters,
result descriptors, obligations and every other field exact. The acceptance
audit is recorded separately in B-006-acceptance.md. Eighteen final Ashlar Rust tests pass; the full workspace checkpoint
passes 182 tests, before the additional declaration assertion.
