# 05-deploy

Qualified registry execution records are retained under `evidence/`; each claim is scoped to its recorded release and gate.

## Public PyPI distribution

The next distribution-name cohort is `umf-weft==0.4.55`; it retains the
`weft` Python import and compiler protocols. PyPI returned 404 for that exact
version and the GitHub tag and release were absent at the 2026-10-26 check.
Those point-in-time checks do not qualify the new archive. The `weft-sql`
0.4.51 and 0.4.54 receipts below remain historical proof for their original
files; the renamed cohort requires its own complete artifact and installed
consumer qualification.

Owner direction on 2026-10-26 requires the qualified Python distribution on
public PyPI, with future released versions published automatically. The
`publish-pypi.yml` driver runs on the default branch and consumes exact existing
GitHub release assets after the release producer succeeds. A normal release
dispatched from `main` hands off its tag, source and checksum manifest in a
run-bound intent artifact; a successful publish job without that artifact
fails visibly. An empty-tag build with a skipped publish job is a no-op.
Selected release producers retain their exact metadata/source pin profiles.
Explicit backfills require the release tag, its source/metadata commit and SHA256SUMS digest.
No existing version is rebuilt, renamed or retagged for the index.

The validator checks tag identity, release pins, GitHub asset digests, checksum
entries, archive names/versions and dependency metadata. A separate main-only
`pypi` environment grants OIDC upload authority to that workflow without stored
tokens or a reviewer per release. A version must be absent or already contain
exactly the same files and bytes. Conflicts and registry errors fail closed;
only initial absence is retried within a bounded visibility wait.

A job without OIDC verifies a fresh public-index installation and retains its
receipt separately from the upload/byte receipt. Configuration and local tests
do not prove publication. Actual OIDC publication, exact public-byte checks and
fresh PyPI consumer gates passed for `weft-sql==0.4.51` in
[run 40358556206](https://github.com/DocumentDrivenDX/weft/actions/runs/40358556206)
and `weft-sql==0.4.54` in
[run 40361089495](https://github.com/DocumentDrivenDX/weft/actions/runs/40361089495).
The original GitHub archives were uploaded unchanged. Source, publication,
consumer and artifact-digest verification records are retained under
`evidence/pypi-0.4.51/` and `evidence/pypi-0.4.54/`. The exact existing
`weft-sql==0.4.53` cohort also passed actual OIDC publication, public-byte checks
and a fresh public-index consumer in [run 40386978860](https://github.com/DocumentDrivenDX/weft/actions/runs/40386978860).
Its [raw receipt record](evidence/pypi-0.4.53/README.md) supplies Ashlar's already
selected dependency without changing that consumer's pins. This changes distribution,
not runtime semantics, platform qualification or the latest-only support policy.

Every public registry upload, including historical backfills, executes in
GitHub Actions bound to the verified immutable tag and GitHub release. Local
preparation and verification do not authorize a local registry upload.
