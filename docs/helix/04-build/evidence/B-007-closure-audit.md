# B-007 closure audit

Determination: incomplete. PR #9 must remain draft until the implementation-plan
exit criteria are proved. This audit preserves the full objective and separates
executed compiler components from release/native support qualification.

| Required gate | Current evidence | Determination and remaining work |
| --- | --- | --- |
| All 30 P0 criteria | US-001 resolution audit, US-002 registration audit, US-003/004 native review, US-005 embedding review, US-006 test-plan review and US-007 application review | Every criterion now has an inspected component path. The matrix does not yet determine full release acceptance; native support and evidence qualification cannot be inferred from candidate passes. |
| Unchanged 636 initial cases | Fresh frontend oracle: 303 resolutions, 333 expected refusals; 13 independent bags; exact retained modules | Executed and reconciled on the frontend. No expected-output alteration or unexplained skip. |
| At least 1000 distinct expanded assertions across matrices | 1200 independently generated relational assertions plus public/backend/runtime/native component cases | Count floor met. Public 1273-case host matrix is seven compiled fixture successes and 1266 refusals, not native target coverage. |
| At least 10000 deterministic properties | Four pinned Proptest generators total 10000; parser text adds 5000; seeded JSON checks separately recorded | Count/seed/library evidence present. No observed failure needs minimization; future failures have configured persistence. |
| Mutations and fuzz/resource exercises | Seven source mutants detected, including type filter/DISTINCT/rounding/presence; parser/JSON generators and actual-host resource cases | Named mechanisms executed. No coverage-guided fuzzing claim is made or required by an invented gate. |
| Every critical supported/refused semantic branch | Named critical-path and emission assertion maps; terminal Linux CI checkpoint at f9101d9: 214 tests; local workspace checkpoint B-007-workspace-218: 218 tests across 35 suites, none ignored/filtered; the subsequent eight-test registry checkpoint retains separate scope | P0 path review present. It is not a proof of every individual supported/refused branch: complete branch-to-assertion accounting remains necessary under TP-001. |
| Both initial adapters have qualified native profiles | Native PostgreSQL 17.9 and observed Databricks receipts; original-byte/layout/custody checks; explicit candidates | Not met. Inventory has no supportedNativeProfiles. The current warehouse is identified as Databricks SQL 2026.39 with concrete u/r build hashes using current_version(); the earlier zero-build 4.2.0 value identifies Spark. Corpus-linked warehouse/settings/domain evidence still needs qualification; candidate realization/capability statuses cannot silently become supported. Production adoption is separate, but that does not erase native qualification. |
| Real support-report audit | 41 synthetic verifier controls; real retained Truss report consistency across 21 scopes/76 cases, independent expectations and 65 ordered comparisons; standard retained receipt/corruption replay | Synthetic controls and real native-data consistency pass. B-007-real-ashlar-report-audit adds seven warehouse/build-linked scopes, 21 successful native statements and one expected overflow refusal with independent unsigned expectations. Exact artifact/model/binding scopes and independent expected rows are reconciled for Truss. Python/browser compiler-artifact joins now cover all 76 cases per host, with sixteen corruption controls and retained runtime identity checks. Fresh B-007-truss-session-native retains the engine, UTF8/C locale, standard-conforming strings and repeatable-read settings from every successful query transaction, plus SQL digests; all 76 artifacts/rows/topology agree with prior receipts. The real report audit now reads and validates that archive, using observed server version/settings in all 21 scopes and retaining its custody hashes. Final profile/producer provenance and host execution qualification remain unproved; these passes do not promote candidate capabilities. |
| Native Python and actual browser | Fresh public/Truss builds and Ashlar host replays, exact metadata, traps/resources, loaded artifact hashes | Observed host component matrix passes. Platform metadata does not qualify every interpreter/OS. |
| Versioned support inventory | Candidate inventory with exact evidence hashes and explicit exclusions | Exists, but remains candidate preparation rather than qualified support inventory. |
| Packaging/release procedures and ownership/license | Native wheel RECORD/content audit, built/packed browser transport and isolated consumer, release procedure | Local packaging checks pass. Owner license and actual release maintainer/registry ownership remain unresolved. The implementation plan requires these decisions before distribution; distribution was not requested, so they are not B-007 merge blockers. Final distributable binaries likewise remain a publication gate. |
| Sequential reviewed PR and merge | B-002–B-006 merged; B-007 PR #9 open/draft | Must merge B-007 only after its gates are satisfied. |

Next compiler work is complete critical branch accounting and a real qualified
support report for each exact claimed profile. Owner release decisions are needed
before distribution, not before this implementation PR can merge. Native tests remain possible against owned synthetic fixtures;
there is no missing Truss table-layout artifact blocking that work. Retesting
already passing components without a new change or a concrete uncovered branch
would not close the outstanding qualification gates.

Current warehouse application evidence now passes all 112 native COUNT/entity/
single/composite keyset cases with 360 integrity checks (488 terminal statements
including 16 empty-page probes). The independent reconciliation passes all 112
expected bags/pages and exact SQL/parameter/pin/metadata custody. Warehouse
identity is observed with 96 nonempty results and by a separate probe after each
of 16 empty pages. All observations are Databricks SQL 2026.39 with the same
retained build hashes. This advances native profile qualification for this
corpus; older compound and relationship scopes are not retroactively pinned.
The standard replay now passes 24 components and 97 hashed references.

Current-build compound entity/page and relationship scopes now have fresh
native executions and independent reconciliation: 48 compound cases and 52
relationship outcomes (28 results, 24 pre-query refusals). The current warehouse
identity is captured with nonempty queries/guards and separately probed after
empty queries. Older receipts remain historical; no retroactive version pinning
is used. Remaining native work is the broader scalar domain/settings evidence
and its registered support qualification, alongside the broader semantic branch
audit. The standard retained replay now includes these fresh scopes.

The original scalar join/SUM corpus and full recursive/scalar/presence value
corpus now also have fresh current-build native evidence and independent
reconciliation: ten scalar outcomes and 133 value outcomes. The read-only
ANSI probe observes true between matching warehouse builds, without an inferred
setting for earlier sessions. The standard replay passes 29 components and
106 hashed references. Registered supported-profile qualification, remaining
numeric/home domain accounting and the broader semantic branch audit remain
open; this is not blanket native support or completed acceptance.

A subsequent numeric-domain checkpoint executes every signed BIGINT width 1–64
on the same identified warehouse. Its 194 terminal statements include 192
successes and two expected CAST_OVERFLOW refusals. Repeated min/max bags sum to
-2, preserving duplicates; guards detect both out-of-range directions and
native null. Independent reconciliation and seven semantic corruption controls
pass. These synthetic owner substitutions prove emitted numeric operations and
guards, without claiming publication/table custody.

Fresh two-adapter Python ABI and browser WASM builds now have actual host byte
parity on 591 retained native-tested requests: 515 Ashlar and 76 Truss, including
the unsigned-width-64 compile refusal. The fresh CLI compares complete artifacts
to original native receipts; only the explicitly obsolete Spark field is removed
from 387 historical Ashlar artifacts. Both actual host receipts join all 591
artifacts (1,182 joins). Source inputs, binary/wheel hashes, package RECORD checks,
compressed original requests/responses, build logs and runtime identities are
retained in B-007-current-initial-hosts. This closes the current-source host
comparison after the metadata correction; it does not claim each host executed
SQL. Registered supported-profile qualification and remaining critical semantic
branch accounting still need completion. The standard replay now passes 34
components and verifies 116 hashed references.

The subsequent full workspace run terminates with 218 passing tests and one
failed test across 22 terminal suites, then stops before the remaining suites.
The failure is a stale runtime expectation for the obsolete Spark
`versionReported` field; this is retained as failed evidence in
B-007-workspace-metadata-failure. The focused regression passes after allowing
only removal of that exact historical field and preserving the complete original
native receipt. A new full workspace run is required; this failure is not
reported as a passing workspace checkpoint.

Explicit `NativeReview` library registrations now pin PostgreSQL 17.9 and
Databricks SQL 2026.39 separately from the historical candidates. Two scoped
registration tests preserve scalar typed/props SQL, parameters, columns and
logical pins, retain existing obligations and add an exact native-profile host
obligation. Wrong versions/profiles and disabled candidate opt-in refuse without
SQL. Both declarations stay candidate, with no qualification evidence claims.
This establishes the versioned engine-profile registration seam needed for the
remaining native-support audit; it does not close that audit or qualify a new
Python/browser composition. Scope and rollback are documented in TD-006.
