# 05-deploy

Qualified registry execution records are retained under `evidence/`; each claim is scoped to its recorded release and gate.

## Public PyPI distribution

`umf-weft==0.4.55` is published on public PyPI, retaining the `weft` Python
import and compiler protocols. Its fresh Linux x86_64, Linux ARM64 and macOS
ARM64 wheels passed the bounded hosted release profile, immutable GitHub
promotion, actual OIDC publication, exact public-byte verification and fresh
Ubuntu public-index installation/import. See the
[release scope](releases/v0.4.55.md) and
[raw distribution proof](evidence/pypi-0.4.55/README.md). Mac compiler parity
covers nine frozen refusal responses; Linux qualification is packaging-only.
Consumer admission and broader compiler conformance remain separate. The
`weft-sql` receipts below retain their original file identities and scope.

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

The Actions/OIDC route above remains the established route for releases that
use it. The explicitly versioned 0.4.56 local profile is a narrow exception to
that execution route: it permits local assembly and, after the separate
reviewed public-transaction steps in its runbook, an owner-authorized local
publication. It does not make local verification itself an upload permission,
change the hosted publisher's requirements, or establish an automatic route
for later versions. The 0.4.56 packet, final release gates, metadata merge,
assembly review and publication authority are still required; until then the
release remains held. See the [0.4.56 release record](releases/v0.4.56.md) and
[local release runbook](local-release-runbook-0.4.56.md).

The local packet validator derives P56/M source correspondence from complete
source and index rosters, checks the selected build-input hashes and P-to-M
ancestry observation, and verifies original per-platform receipt chains and
wheel members. Required gate receipts are pinned to reviewed actual bytes;
unknown or unfinished final gates remain held. The final asset review is
performed after assembly and kept outside the evidence ZIP. The local profile
does not change ordinary UMF AST 0.1: Weft's explicitly selected module
envelope 0.2 admits only a matching non-null literal default and preserves
omitted-input behavior.

The actual Linux source/package-guard result is accepted only for its recorded
P56 source, Python 3.11.17/Bun 1.4.3 tool profile, and measured complete output
of 32,866 bytes. Its operator configured a 4 MiB raw-output threshold; that
configuration is not approved as a future limit. The measured result remains
below the original 1 MiB bound. Exact Linux x86_64 and ARM64 static-wheel
artifacts and the macOS ARM64 build/static-only chain have separate accepted
reviews. The Linux native run's original failure is preserved alongside its
separate accepted predicate reanalysis; neither implies an additional runtime
execution. Inherited-boundary and browser/WASM gates remain open.
