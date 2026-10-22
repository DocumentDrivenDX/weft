# Selected Weft 0.4.51 workspace qualification

This packet records full local Rust workspace tests at selected source commit
`56d5507c6b0169574277661dd85d8c5feb22e6a7`. Runs used macOS ARM64, Rust/Cargo
1.90.0, `--locked --offline -j2`, `CARGO_INCREMENTAL=0`, and the separate target
`/private/tmp/weft-arras-051-full-target`. `producer-custody.json` records exact
commands, source identities, profile settings, results, and each log's original
path, byte count and SHA-256.

## Completed selected profiles

The candidate profile passed with 1,046 tests across 62 Rust test-result
summaries, with no failures or ignored tests. It used Cargo's default test
profile; the three `CARGO_PROFILE_TEST_*` variables were not overridden.

The qualified profile passed with 1,047 tests across 62 summaries, with no
failures or ignored tests. It used the selected conformance workflow's test
profile: `CARGO_PROFILE_TEST_OPT_LEVEL=1`,
`CARGO_PROFILE_TEST_DEBUG_ASSERTIONS=true`, and
`CARGO_PROFILE_TEST_OVERFLOW_CHECKS=true`.

Before the full workspace runs, focused `weft-runtime` candidate and qualified
profiles also passed, with 21 and 22 tests respectively across 10 result
summaries each. The full workspace logs include the repaired B-001 boundary
test `numeric_corpus_uses_independent_expected_domains`: all 636 original UMF
0.7.0 cases remain in `fixtures/cases-umf070.json`, whose SHA-256 is
`5d13b1d0d77074193e678321406f57a9ce34e49cd413649f0f7f7210262c6306`; its 575
numeric expectations remain 288 compiled and 287 blocked.

The candidate and qualified results are local macOS ARM64 Rust workspace
evidence with explicitly stated test profiles. Linux native/Chromium reports
are separate; see [the successful Linux parity gate](../release051-final-integration-39822650070-20261022/README.md).
The separate full-conformance workflow `39822650250` is not represented by
these local runs and remains a distinct closeout item.

## Earlier attempts retained

The packet retains the original pre-fix candidate failure and the safely
interrupted, superseded qualified run. The original candidate run exposed the
stale test expectation for historical B-006 UMF 0.7 input (`WFT-INPUT` in the
selected compiler). The test-only expectation was corrected without changing
production code or the historical fixture. Root stopped the earlier qualified
run only after identifying and terminating its exact process group; that
partial run is not qualification evidence. Their logs and precise status are
preserved separately from the passing selected-source runs.

The tested source commit follows the selected compiler freeze
`86c092178bb8ca1a0d84edfebc175009d8862493` with the original B-001 test fixture
restoration and a focused `cfg(test)` expectation correction. No compiler
production implementation, Cargo manifest/lockfile, vendor, specification,
toolchain, or contract input was altered by those fixes. A later evidence-only
commit carries this packet; the test identities remain bound to the source
commit above.
