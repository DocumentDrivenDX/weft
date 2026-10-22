# Weft 0.4.51 source-F evidence

This packet records the current selected compiler/build source freeze
`557153731e9f5d0583193a1189a07c6ed485d3d3` (source F). It keeps local workspace,
Mac wheel, Linux packaging and hosted conformance evidence distinct. The
source-F Mac native module is different from the earlier `a21` candidate, so
prior native functional results are not inherited. Original 86/616 packets and
the existing `local-macos-0.4.51.json` remain unchanged historical evidence.

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
module. The new Linux run contributes packaging evidence only; no source-F Linux
runtime corpus result is claimed. Formal and mutation receipts are retained
with their stated scopes; mutation evidence does not claim native
mutated-result qualification.

## Still pending

Source-F Truss host qualification is open. In particular, this packet does not
claim the new installed-wheel host checks for 80 lineage cases, 62 independent
cases, 10 policy controls or the 1,000-object workload. The first new-host
attempt's PostgreSQL shared-memory permission failure is retained by the host
campaign owner; its permission-correct rerun is still required. The previous
`local-macos-0.4.51.json` has not been rewritten and must not be treated as the
current source-F qualification. No publication or release authorization is
asserted here.

## Included evidence

- `producer-custody.json`: original paths, lengths and SHA-256 digests for each
  copied report, log, manifest, event stream and receipt. No cursor or private
  key files were read or copied.
- `macos/`: source-F build custody, exact wheel verification/comparison, actual
  module test reports and bounded build/runtime logs.
- `linux/`: hosted packaging run, per-platform reports and independent checks.
- `hosted-ci/`: run identity and the exact public formal, fresh-host, mutation
  manifests/events/qualifications and host parity receipts.

The two large generated resource-control input files and built CI wheel/WASM
payloads are not duplicated here; their exact sizes and hashes are retained in
the copied source receipts or custody inventory where applicable.
