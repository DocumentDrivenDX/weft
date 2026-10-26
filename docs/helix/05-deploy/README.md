# 05-deploy

No governed execution artifacts yet. Add release or iteration records when work reaches this activity; do not imply deployment or implementation evidence from the bootstrap.

## Public PyPI distribution

Owner direction on 2026-10-26 requires the qualified Python distribution on
public PyPI, with future released versions published automatically. The
`publish-pypi.yml` driver runs on the default branch and consumes exact existing
GitHub release assets after the release producer succeeds. A normal release
dispatched from `main` hands off its tag, source and checksum manifest in a
run-bound intent artifact; a successful publish job without that artifact
fails visibly. An empty-tag build with a skipped publish job is a no-op.
Selected release producers retain their exact metadata/source pin profiles. Explicit backfills
require the release tag, its source/metadata commit and SHA256SUMS digest.
No existing version is rebuilt, renamed or retagged for the index.

The validator checks tag identity, release pins, GitHub asset digests, checksum
entries, archive names/versions and dependency metadata. A separate main-only
`pypi` environment grants OIDC upload authority to that workflow without stored
tokens or a reviewer per release. A version must be absent or already contain
exactly the same files and bytes. Conflicts and registry errors fail closed;
only initial absence is retried within a bounded visibility wait.

A job without OIDC verifies a fresh public-index installation and retains its
receipt separately from the upload/byte receipt. Configuration and local tests
do not prove publication. The approved environment and trusted publisher are
configured; the qualified backfill remains pending; actual public-byte and install receipts must pass before the
public-index install claim is made. This changes distribution, not runtime
semantics, platform qualification or the latest-only support policy.
