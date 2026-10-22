# Weft 0.4.51 source-F evidence

This packet records the current selected compiler/build source freeze
`557153731e9f5d0583193a1189a07c6ed485d3d3` (source F). It keeps local workspace,
Mac wheel, Linux packaging and hosted conformance evidence distinct. The
source-F Mac native module is different from the earlier `a21` candidate, so
prior native functional results are not inherited. Original 86/616 packets remain
unchanged historical evidence. The prior source-86 macOS JSON is preserved in
`superseded/`; `local-macos-0.4.51.json` now records source-F results only.

## Selected source tests

The full selected workspace tests ran at `56d5507c6b0169574277661dd85d8c5feb22e6a7`,
with no changes to protected compiler/build inputs relative to F. The default
candidate profile passed 1,046 tests; the qualified CI test profile passed
1,047. Each run covered 62 result summaries with zero failures or ignored tests.
The exact commands, source inputs, logs and profile settings remain in the
[selected workspace packet](../release051-selected-workspace-56d5507c-20261022/README.md).

## New Mac source-F wheel and module checks

The Mac ARM64 wheel was rebuilt from F using Python 3.11.17, Rust 1.90.0 and
maturin 1.9.6. Its archive SHA-256 is
`e60e552f02c670b43a43111b08d26d6b415aaa9b9de42f241c37ad1d9fdd2487`; its native
module SHA-256 is
`34691932af5b84a3e8629b318aa69e88a5e2df46e47ffc143b4950aa6defc5aa`. The exact
installed-wheel packaging verifier passed. With this actual new module, the
selected runtime library suite passed 51 tests, entity selection passed all 33
cases, and facts passed all 5 cases. Both raw reports were byte-identical to
their independent expectations. The prior module hash was
`93bee228350743286073a9400bdf5ec92e387c5e35431bbf34b0f81279c460a2`; it differs
from the new module and supplies no transferred evidence.

## Linux source-F packaging

Packaging run [39830424802](https://github.com/DocumentDrivenDX/weft/actions/runs/39830424802)
and its independent archive verifier passed at F for both Linux x86_64 and
aarch64 wheels. The x86_64 wheel SHA-256 is
`a23142f12b59444138b588a8a764666b025552a14036f62d83af022153f1a0a5`, and its
native module SHA-256 is
`ef293922003832fbd87041a28c6dbde33927fee573270f203aa51daf06bcd445`. The
aarch64 wheel was packaged and archive-checked, but this packet does not admit
that platform or claim its runtime behavior. Linux packaging is not Linux
native database or Truss host execution evidence.

## Hosted final conformance workflow

Hosted conformance run [39822650250](https://github.com/DocumentDrivenDX/weft/actions/runs/39822650250)
completed successfully with all source jobs and steps passing. It used the
synthetic testability source `b0808e8c` and workflow head `cc694344`; PR 27 later
merged as `3cd05d5389375b7d40622821d4788828135a2d6e`. The merged tree and tested
synthetic source tree are both
`d5d7986179223d703c431dc77c829f851ba1a524`. These workflow/testability
identities are recorded separately from release source F.

The included fresh-host receipt is explicitly limited to 2,181 unchanged
retired UMF 0.7 inputs refused by the current envelope, plus resource/security
and Python/WASM parity controls. It does not report successful current UMF 0.10
Truss compilation or database execution. The separate current compiler corpus
is 33 entity-selection and 5 facts cases, reported above against the new Mac
module. The source-F Linux packaging run alone contributes no runtime corpus
qualification; public Linux Truss host evidence is recorded below. Formal and
mutation receipts are retained
with their stated scopes; mutation evidence does not claim native
mutated-result qualification.

## Source-F Truss host qualification

The exact installed Truss0.0.48 / Weft0.4.51 ARM64 host gates passed on local
macOS with PostgreSQL18.6: 80 migration-lineage tests, 62 independent archive
cases, six cohort/asset-cache tests, 14 policy executions across 10 distinct
controls, and the 1,000-object/1,000-edge public workload. Four policy
executions repeat imported query tests; they are not four additional distinct
controls. The final serialized policy retry passed with the same 2-second SQL
statement and 30-second operation bounds. Earlier skipped and timeout attempts
remain in the packet as unqualified history. The first migration attempt refused
on the runner's shared-memory permission; only the permission-correct retry
counts as evidence.

The exact copied Truss reports, commands, package/environment correspondence
and prior-attempt outputs are under [`truss-source557/`](truss-source557/);
`evidence-inventory.json` records their original paths, lengths and SHA-256
values. It binds runtime source `22ed333…`, its 455-file source/wheel/sdist
correspondence, Weft source F, wheel/module hashes, native/migration manifests
and the refreshed independent archive. The comprehensive Truss-owned packet is
maintained separately.

The exact original source-86 JSON is preserved as
[`superseded/local-macos-0.4.51-source86-superseded.json`](superseded/local-macos-0.4.51-source86-superseded.json)
with SHA-256 `29d71220bb593ecc85c89e8a9f2c7fa9967f97adf98dfa3e85c4c3988e9409d3`;
its counts and source identity are historical only. Source-F local macOS
qualification is recorded in the active
[`local-macos-0.4.51.json`](../../releases/local-macos-0.4.51.json). The published `v0.4.51` release is independently verified. Its public release
metadata, asset inventory and exact verifier receipt are under
[`public-release/`](public-release/); a clean install of the public Mac wheel also
passed packaging verification, 33 entity cases and 5 facts cases. The Weft release
is public; separate Truss release status and Linux host execution are not inferred
from this packet.

## Included evidence

- `producer-custody.json`: original paths, lengths and SHA-256 digests for each
  copied report, log, manifest, event stream and receipt. No cursor or private
  key files were read or copied.
- `macos/`: source-F build custody, exact wheel verification/comparison, actual
  module test reports and bounded build/runtime logs.
- `linux/`: hosted packaging run, per-platform reports and independent checks.
- `hosted-ci/`: run identity and the exact public formal, fresh-host, mutation
  manifests/events/qualifications and host parity receipts.
- `truss-source557/`: exact local installed-host receipts for the five passing
  Truss gate groups, source/environment correspondence and unqualified prior
  attempts; the inventory binds each copied file to its original path and hash.
- `public-release/`: exact public release metadata, six non-wheel public assets,
  all-nine-asset verification and clean public Mac install reports/logs. No wheel
  payload is duplicated.

The two large generated resource-control input files and built CI wheel/WASM
payloads are not duplicated here; their exact sizes and hashes are retained in
the copied source receipts or custody inventory where applicable.

## Public release closeout

Weft 0.4.51 is published at [GitHub releases](https://github.com/DocumentDrivenDX/weft/releases/tag/v0.4.51). The strict publisher run was [39838369924](https://github.com/DocumentDrivenDX/weft/actions/runs/39838369924), with tag source `d4b0b98507ae15bd98180a2ea2030ea92ce11d96` and frozen source F `557153731e9f5d0583193a1189a07c6ed485d3d3`. The independent verifier passed all nine release assets: three wheels, three packaging reports, the Mac qualification, SHA256SUMS and release pins. The public [SHA256SUMS manifest](https://github.com/DocumentDrivenDX/weft/releases/download/v0.4.51/SHA256SUMS) is 394 bytes with SHA-256 `62c9d8082eb2f446446d7b94ff7ef7f48769e3cd836e5641c2951069302a6c75`; the copied manifest matches those exact bytes. It checked release metadata and asset sizes, all three wheel checksums, ZIP and RECORD membership/digests, native module bytes, Linux report correspondence, Mac report correspondence and release-pin identities.

A fresh clean public macOS install passed wheel packaging verification plus all 33 entity and 5 facts cases. These reports match the source-F Mac evidence bytes. The Linux aarch64 wheel is present and packaging-verified; no aarch64 Truss host or database execution is claimed. The local Truss host results above remain limited to macOS ARM64 and PostgreSQL18.6. The public verification and fresh-install receipts are copied without wheel payloads under `public-release/`.

The public [Truss 0.0.48 release](https://github.com/DocumentDrivenDX/truss/releases/tag/v0.0.48)
also completed its mandatory Linux x86_64 tag qualification against the public
Weft 0.4.51 source-F runtime pair. Tag run
[39838914360](https://github.com/DocumentDrivenDX/truss/actions/runs/39838914360)
passed: 80 migration-lineage tests, 62 independent conformance cases and the
1,000-object/1,000-edge workload. The public handoff verifies the exact Truss
wheel and binds compiler source F, native module
`ef293922003832fbd87041a28c6dbde33927fee573270f203aa51daf06bcd445`, and
original Linux workflow evidence. This evidence qualifies the stated latest-only
Linux x86_64 host scope; it does not qualify Linux aarch64 database execution.
The three public verification receipts are preserved in the
[Linux evidence inventory](public-release/truss-linux-x86_64/evidence-inventory.json), with
original paths, byte lengths and SHA-256 values in its inventory. No wheel
payload is duplicated.
