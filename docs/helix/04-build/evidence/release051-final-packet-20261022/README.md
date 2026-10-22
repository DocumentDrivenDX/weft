# Weft 0.4.51 selected release evidence

This packet preserves the selected release-source evidence and separately scoped
Linux producer outputs. Original producer JSON is copied byte-for-byte; the
`producer-custody.json` wrapper records each original path, byte length, SHA-256,
and the source identity used for the evidence. It does not replace producer
reports or convert an incomplete workflow into a passing result.

## Linux packaging

GitHub Actions packaging run `39811059843` completed successfully at
`616aa7b6b7fc7b45a53a2bf51751500935312e84`. The independent verifier passed for
the aarch64 and x86_64 Linux wheels, checking the actual wheel archives, full
RECORD contents and correspondence between packaged native modules and source
reports. Both per-platform reports identify the same commit as the reused run's
`headSha`. The verifier and original reports are included here. This is packaging
and artifact-integrity evidence; it does not claim local Linux execution.

## Scoped Linux native and browser reports

Parity run `39811016270` at
`c5a4537db6ba54e52a988c0f05b5df69d09991dd` completed successfully. It supplied
separate Linux reports for 33 entity-selection cases and 5 facts cases, together
with their Chromium receipts. The receipts report expected/actual byte parity and
use Chromium `151.0.7922.34` with Playwright `1.62.1`. Both the original
in-progress run snapshot and the later completed-run metadata are included so
that the transition is visible. The distinct full-conformance workflow
`39822650250` remains in progress; these parity reports do not close that gate or
full integration qualification.

The browser summaries identify a 28,344,868-byte Linux debug-profile WASM module
with SHA-256
`b1a573acb81e45e345357dc8b6820d6765479c17827b9d7a8b689957c141266f`. It is
distinct from the 9,740,400-byte selected macOS release-profile module with SHA-256
`73012141c317c5eea4feabe81f448b8df832822267cd3ca83b4b1420fe875d30`, recorded in
the local macOS qualification. These are platform/profile-specific artifacts and
are not interchangeable evidence.

## Table-operation reports

The selected binding report is tied to commit
`6636e8f60cc521810afbc71394ce8141666bdae9` and contains 56 cases: 24 compiled and
32 blocked. The native report explicitly scopes its results to candidate SQL
behavior under source-installed PostgreSQL 18 RLS; its `nativeExecutionQualified`
field is false. The browser report records Chromium parity evidence and explicitly
states `nativeExecutionQualified: false`; neither report qualifies production
execution. These reports do not establish generated/runtime operation admission,
production executor behavior, or a release qualification for the operation
surface.

## Included files

- `packaging-run-39811059843.json`, the two platform reports, and
  `packaging-independent-verification.json`: final Linux package provenance and
  independent artifact verification.
- `linux-parity-run-39811016270-snapshot.json` and
  `linux-parity-run-39811016270-completed.json`, facts/entity reports, and browser
  summaries/receipts: scoped Linux reports and both workflow status snapshots.
- `operations-bindings.json` and the native/browser candidate reports: selected
  table-operation evidence with its bounded scope.
- `producer-custody.json`: exact source paths, byte counts, hashes, run/commit
  identities, and the distinct macOS/Linux browser module identities.

The separate full-conformance workflow `39822650250` remains in progress. The
selected source freeze remains
`86c092178bb8ca1a0d84edfebc175009d8862493`. This packet does not authorize or
perform publication, and the open integration workflow remains a release-closeout
item.
