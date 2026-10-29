---
ddx:
  id: weft.local-release-runbook-0.4.56
  type: runbook
  activity: deploy
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.release-0.4.56
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# Weft 0.4.56 local release runbook

This runbook describes a narrow local release path for version 0.4.56; it
does not dispatch workflows. Before a GitHub write, the publisher checks the
five exact automatic workflows are `disabled_manually`; global Actions and
manual workflows need not be disabled. It is not a qualification receipt or
authorization to publish. The local assembler is offline by default and accepts only a closed
`weft-local-build-static-release/1` packet whose exact source, artifacts,
final gates, and pre-assembly source/artifact reviews are present and accepted.
The independent review of the final asset directory is a later sidecar input
to publication and is deliberately outside the immutable evidence archive.

## Required inputs

The packet must bind producer P56 (`973d864c8d26b2bc173a411eac8368da5efc1ab3`,
tree `8b20d1f01bea72ea08ed01fcaac97266ed3cb4ff`) separately from release
metadata commit M. M must contain the reviewed release metadata and preserve
the exact producer build inputs. The packet must retain the actual local
operator Q source and execution identities, including their bounded command
records and retained log hashes; it must not invent a GitHub run, hosted
attestation, or Actions artifact identity. The actual UMF test source is a
separate identity from Weft producer P56, and the release metadata commit M is
validated from its source map and ancestry observation, not a packet boolean.
The closed source-map/2 form stores P's complete sorted Git stage-row roster;
the fixed 1,459-input projection is derived from the same pinned source-package
receipt already required by the packet. M uses a single complete seven-path
delta against that exact P map. The verifier reconstructs the logical maps and
rechecks full roster, selected-input, source-blob and ancestry digests. The
source-guard reference must be identical to the packet's source-package gate
receipt; no selector or base path is supplied independently.

The three original wheel byte streams must be exactly the x86_64 Linux,
ARM64 Linux, and ARM64 macOS `cp39-abi3` wheels for `umf-weft==0.4.56`. Their
static member, WHEEL/RECORD, native architecture and source correspondence must
match the actual reviewed receipts. The final installed-admission, native
scenario, inherited-boundary, and platform-static evidence must bind those exact
bytes. The original x86_64 native operator receipt remains `FAIL_OR_UNKNOWN`;
a separately reviewed pure predicate reanalysis accepts the 14 case
predicates across its 19 retained scenario records without rewriting the
original failure or claiming a rerun. The x86_64 build failure and separate
static-audit result also remain separate records. Accepted static macOS
artifact evidence does not qualify Mac runtime behavior.

Missing, stale, failed, unknown, or unreviewed evidence keeps the packet held.
Do not substitute earlier prototype wheels, version-only matches, a source
plan acceptance, or a prior release's platform result.

## Offline assembly

After the final source merge M and packet review, assemble into a new empty
directory. The verifier checks bounded regular files, exact receipt hashes,
complete source identities, actual receipt contents, the required gate roster,
wheel metadata and members before copying original wheel bytes. The generated inventory
and `SHA256SUMS` exclude themselves as documented; the evidence index excludes
its own bytes. The evidence archive contains bounded source, operator, gate,
review and log evidence, never software wheels. The 16 MiB evidence limit and
70 MiB complete-assembly limit are checked before copying; any cap overflow or
changed byte stops assembly.

The command interface is:

```sh
python3 scripts/release/publish_local_release.py assemble \
  --packet /path/to/reviewed-local-packet.json \
  --notes docs/helix/05-deploy/releases/v0.4.56.md \
  --metadata-review-sha256 <independent-review-digest-for-actual-M-source-map-and-ancestry> \
  --output /new/empty/release-directory
```

The command must return `ASSEMBLED_LOCAL_UNSIGNED`. Preserve its output and
review it independently before any public write. Do not overwrite a prior
output; preserve failed or partial directories for diagnosis.

The inherited-boundary adapter consumes the original Table012 receipt and
review schema. It retains the complete 251-row public metadata roster and
receipt while carrying the five selected Table runtime artifacts as separately
exactly ten hashed external-software witnesses. Assembly rejects an incomplete
roster or any payload inclusion; packet-controlled test flags cannot disable the
production check. The browser and WASM obligations both
bind the same original accepted Truss011 receipt and review. Its full 133-row
output roster is closed, but the five selected runtime files remain outside
the evidence ZIP; the other 128 metadata rows and original receipt/review are
included. The verifier rehashes the external files against the pinned source
root and proves that their payload bytes were not placed in the ZIP. They are
not wheel-family bytes or evidence payloads, and this mapping does not itself
create a READY packet or complete the held M-source/release review.

The M-source review digest is supplied from an independently reviewed source
correspondence record outside M and outside the packet. After assembly, the
separate final-review record must bind the exact `SHA256SUMS` digest and
`asset-inventory.json` digest; provide that review and its independently
recorded digest to every public-phase command as `--review` and
`--review-sha256`. Neither digest is self-selected by the packet.

## Public transaction gates

Public publication is a separate owner-run transaction after assembly review
and the normal metadata merge. First create the exact numeric draft, upload
the closed nine-asset family, verify numeric API inventory and downloaded
bytes, and only then publish the draft. Recheck that tag `v0.4.56` and its release do
not already exist, confirm the exact M target and frozen `SHA256SUMS`, then
create only the numeric draft with the closed asset set. Verify the numeric
release identity, exact asset names, sizes, and downloaded bytes before
promoting the draft; verify server immutability after publication. Never overwrite a conflicting tag
or version; reconcile an unknown write outcome through exact server readback.

PyPI publication requires a project-owner-authorized supported upload
credential and a pinned, source-reviewed local upload client. No credential
value may appear in argv, logs, packets, or evidence. Do not search unrelated
secret stores or claim the GitHub token supplies PyPI authority. If a supported
credential or project policy is unavailable, hold only the authenticated
upload execution; the transport remains implemented and mock-testable. The
local uploader software manifest has a separate preflight bound of 32 MiB and
2048 regular files, with the existing 16 MiB per-file limit. This allowance
does not change the 16 MiB release-evidence limit, 1 MiB raw-capture limit, or
70 MiB complete-assembly limit. An upload attempt is not proof: read back the
public version and all three exact served archive bytes, then perform the
separately bounded fresh Linux x86_64
public-index install/admission proof against the actual final tuple.

No step dispatches Actions, triggers a tag build, rebuilds or renames an
archive, or asserts OIDC provenance. PyPI attestation absence, unsigned local
provenance, platform scope, and any remaining gate must be reported plainly.
The separate `readback` phase performs unauthenticated public GitHub tag,
immutable-release, exact-asset-roster, and served-byte checks; it does not
publish or modify the release. GitHub and PyPI writes require their respective
explicit phase, a reviewed local assembly, an independently pinned final
review, and the relevant owner-authorized credential.
The scoped compiler change is Weft module envelope 0.2 for matching non-null
literal defaults; ordinary UMF AST declarations remain 0.1, omitted values are
not synthesized into input, and native-only temporal behavior is unchanged.
